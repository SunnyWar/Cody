import os
import re
import shutil
import subprocess
from datetime import datetime
from pathlib import Path

HUNK_HEADER_RE = re.compile(r"^@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@")

def _extract_warning_signature(clippy_output: str) -> str:
    """Extract a unique signature from clippy output to track attempted warnings.
    
    Returns a string like 'file.rs:78:error:too_many_arguments' or empty string if none found.
    """
    if not clippy_output:
        return ""
    
    lines = clippy_output.splitlines()
    for i, line in enumerate(lines):
        # Look for error/warning lines
        if line.strip().startswith("error:") or line.strip().startswith("warning:"):
            # Next line often has --> file:line:col
            if i + 1 < len(lines) and "-->" in lines[i + 1]:
                arrow_line = lines[i + 1].split("-->", 1)[1].strip()
                # Extract file and line number
                parts = arrow_line.rsplit(":", 2)
                if len(parts) >= 2:
                    file_path = parts[0].strip()
                    line_no = parts[1].strip()
                    
                    # Extract error type from first line
                    error_text = line.strip()
                    # Signature: file:line:error_type
                    return f"{file_path}:{line_no}:{error_text[:50]}"
    
    return ""

def _validate_utf8_in_diff(diff_content: str) -> tuple[bool, str]:
    """Check if diff content contains valid UTF-8 characters.
    
    Returns (is_valid, error_message).
    This catches cases where LLM generates invalid UTF-8 bytes in comments/strings.
    """
    try:
        # Try to encode as UTF-8 to confirm all characters are valid
        diff_content.encode('utf-8')
    except UnicodeEncodeError as e:
        return (False, f"Diff contains invalid UTF-8: {str(e)}")
    
    # Additional check: scan for common signs of corrupted text
    # (e.g., replacement characters, control characters)
    suspicious_chars = [
        '\ufffd',  # Unicode replacement character
        '\x00',     # Null byte
    ]
    for char in suspicious_chars:
        if char in diff_content:
            return (False, f"Diff contains suspicious character: {repr(char)}")
    
    return (True, "UTF-8 validation passed")

def _sanitize_diff(diff_content: str) -> str:
    """Clean up diff format if LLM generated non-standard formatting.
    
    Handles:
    - *** Begin/End Patch markers
    - Missing --- +++ headers
    - Improper @@ markers
    """
    normalized = diff_content.replace("\r\n", "\n").replace("\r", "\n")
    lines = normalized.strip().split("\n")
    
    # Remove *** Begin Patch and *** End Patch markers
    lines = [l for l in lines if not l.startswith('*** Begin') and not l.startswith('*** End')]
    
    cleaned = []
    for line in lines:
        # Convert *** markers to --- +++ format
        if line.startswith('*** Update File:'):
            filename = line.replace('*** Update File:', '').strip()
            cleaned.append(f'--- a/{filename}')
            cleaned.append(f'+++ b/{filename}')
        else:
            cleaned.append(line)
    
    sanitized = "\n".join(cleaned).rstrip("\n") + "\n"
    return sanitized

def _ensure_logs_dir(repo_path: str) -> str:
    """Ensure .cody_logs directory exists and return its path."""
    logs_dir = os.path.join(repo_path, ".cody_logs")
    os.makedirs(logs_dir, exist_ok=True)
    return logs_dir

def _save_diagnostic(logs_dir: str, name: str, content: str) -> None:
    """Save a diagnostic file with timestamp."""
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    filename = f"{timestamp}_{name}.log"
    filepath = os.path.join(logs_dir, filename)
    with open(filepath, "w") as f:
        f.write(content)
    print(f"[cody-graph] [DIAG] Saved: {filename}", flush=True)

def _run_patch_with_strategies(repo_path: str) -> subprocess.CompletedProcess:
    """Try multiple git apply strategies, including zero-context hunks.

    LLM-generated diffs frequently resemble --unified=0 format (no unchanged
    context lines), which can fail with plain `git apply`.
    """
    strategies = [
        ["git", "apply", "--whitespace=nowarn", "changes.patch"],
        ["git", "apply", "--whitespace=nowarn", "--unidiff-zero", "changes.patch"],
        ["git", "apply", "--whitespace=nowarn", "--unidiff-zero", "--recount", "changes.patch"],
    ]

    last_result = None
    for idx, cmd in enumerate(strategies, start=1):
        print(f"[cody-graph] [DIAG] git apply strategy {idx}/{len(strategies)}: {' '.join(cmd)}", flush=True)
        result = subprocess.run(
            cmd,
            cwd=repo_path,
            capture_output=True,
            text=True,
        )
        last_result = result
        if result.returncode == 0:
            print(f"[cody-graph] [DIAG] Strategy {idx} succeeded", flush=True)
            return result
        print(
            f"[cody-graph] [DIAG] Strategy {idx} failed with exit code {result.returncode}",
            flush=True,
        )

    return last_result

def _normalize_diff_path(path: str) -> str:
    path = path.strip()
    if path.startswith("a/") or path.startswith("b/"):
        path = path[2:]
    return path.replace("\\", "/")

def _parse_unified_diff(diff_content: str) -> list[dict]:
    lines = diff_content.splitlines()
    patches: list[dict] = []
    i = 0
    while i < len(lines):
        line = lines[i]
        if not line.startswith("--- "):
            i += 1
            continue

        if i + 1 >= len(lines) or not lines[i + 1].startswith("+++ "):
            raise ValueError("Malformed diff: missing +++ header")

        old_path = _normalize_diff_path(line[4:])
        new_path = _normalize_diff_path(lines[i + 1][4:])
        i += 2

        hunks: list[dict] = []
        while i < len(lines):
            if lines[i].startswith("--- "):
                break
            if not lines[i].startswith("@@ "):
                i += 1
                continue

            header = lines[i]
            match = HUNK_HEADER_RE.match(header)
            if not match:
                raise ValueError(f"Malformed hunk header: {header}")

            old_start = int(match.group(1))
            old_count = int(match.group(2) or "1")
            new_start = int(match.group(3))
            new_count = int(match.group(4) or "1")
            i += 1

            hunk_lines: list[str] = []
            while i < len(lines):
                hline = lines[i]
                if hline.startswith("@@ ") or hline.startswith("--- "):
                    break
                hunk_lines.append(hline)
                i += 1

            hunks.append(
                {
                    "old_start": old_start,
                    "old_count": old_count,
                    "new_start": new_start,
                    "new_count": new_count,
                    "lines": hunk_lines,
                }
            )

        patches.append({"old_path": old_path, "new_path": new_path, "hunks": hunks})

    if not patches:
        raise ValueError("No file patches found in diff")

    return patches


def _summarize_diff(patches: list[dict]) -> dict:
    file_count = len(patches)
    hunk_count = 0
    additions = 0
    removals = 0
    touched_files: list[str] = []

    for patch in patches:
        touched_files.append(patch.get("new_path") or patch.get("old_path") or "unknown")
        for hunk in patch.get("hunks", []):
            hunk_count += 1
            for hline in hunk.get("lines", []):
                if not hline:
                    continue
                marker = hline[0]
                if marker == "+":
                    additions += 1
                elif marker == "-":
                    removals += 1

    return {
        "file_count": file_count,
        "hunk_count": hunk_count,
        "additions": additions,
        "removals": removals,
        "changed_lines": additions + removals,
        "touched_files": touched_files,
    }


def _validate_diff_policy(diff_content: str, state: dict) -> tuple[bool, str, dict]:
    try:
        patches = _parse_unified_diff(diff_content)
    except Exception as e:
        return (False, f"Invalid unified diff: {e}", {})

    summary = _summarize_diff(patches)
    phase = state.get("current_phase", "clippy")

    # Universal rule: NEVER allow suppression attributes
    suppression_patterns = [
        "#[allow(",
        "#[warn(",
        "#![allow(",
        "#![warn(",
    ]
    for patch in patches:
        for hunk in patch.get("hunks", []):
            for hline in hunk.get("lines", []):
                if not hline or hline[0] != "+":
                    continue
                content = hline[1:].strip()
                for pattern in suppression_patterns:
                    if pattern in content:
                        return (
                            False,
                            f"Rejected diff: adds suppression attribute '{pattern}'. "
                            "Orchestration policy: ALWAYS fix root cause, NEVER hide warnings/errors.",
                            summary,
                        )

    if phase == "clippy":
        if summary["file_count"] != 1:
            return (
                False,
                f"Rejected clippy diff: expected exactly 1 file, got {summary['file_count']}",
                summary,
            )
        if summary["hunk_count"] > 2:
            return (
                False,
                f"Rejected clippy diff: too many hunks ({summary['hunk_count']})",
                summary,
            )
        if summary["changed_lines"] > 30:
            return (
                False,
                f"Rejected clippy diff: too many changed lines ({summary['changed_lines']})",
                summary,
            )

    return (True, "Diff policy check passed", summary)

def _apply_patch_to_lines(path: str, original_text: str, hunks: list[dict]) -> str:
    original_lines = original_text.splitlines()
    has_trailing_newline = original_text.endswith("\n")
    out_lines: list[str] = []
    src_idx = 0

    for hunk in hunks:
        target_idx = max(0, hunk["old_start"] - 1)
        if target_idx < src_idx:
            raise ValueError(f"Overlapping hunks in {path}")

        out_lines.extend(original_lines[src_idx:target_idx])
        src_idx = target_idx

        for hline in hunk["lines"]:
            if not hline:
                raise ValueError(f"Malformed empty hunk line in {path}")

            marker = hline[0]
            content = hline[1:]

            if marker == " ":
                if src_idx >= len(original_lines) or original_lines[src_idx] != content:
                    raise ValueError(f"Context mismatch in {path} at source line {src_idx + 1}")
                out_lines.append(content)
                src_idx += 1
            elif marker == "-":
                if src_idx >= len(original_lines) or original_lines[src_idx] != content:
                    raise ValueError(f"Removal mismatch in {path} at source line {src_idx + 1}")
                src_idx += 1
            elif marker == "+":
                out_lines.append(content)
            elif hline.startswith("\\ No newline at end of file"):
                continue
            else:
                raise ValueError(f"Unsupported hunk marker '{marker}' in {path}")

    out_lines.extend(original_lines[src_idx:])
    result_text = "\n".join(out_lines)
    if has_trailing_newline:
        result_text += "\n"
    return result_text

def _matches_subsequence(lines: list[str], start: int, expected: list[str]) -> bool:
    if start < 0 or start + len(expected) > len(lines):
        return False
    return lines[start:start + len(expected)] == expected

def _find_unique_subsequence(lines: list[str], start: int, expected: list[str]) -> int | None:
    if not expected:
        return start

    matches: list[int] = []
    max_start = len(lines) - len(expected)
    for idx in range(max(0, start), max_start + 1):
        if _matches_subsequence(lines, idx, expected):
            matches.append(idx)
            if len(matches) > 1:
                return None

    return matches[0] if matches else None

def _apply_unified_diff_python(repo_path: str, diff_content: str) -> tuple[bool, str]:
    patches = _parse_unified_diff(diff_content)
    for patch in patches:
        target_path = patch["new_path"] if patch["new_path"] != "/dev/null" else patch["old_path"]
        abs_path = os.path.join(repo_path, target_path)
        if not os.path.exists(abs_path):
            return (False, f"Target file not found: {target_path}")

        with open(abs_path, "r", encoding="utf-8") as f:
            original_text = f.read()

        # Apply with line-number guidance, but tolerate stale hunk locations by
        # searching for a unique old-content subsequence when needed.
        updated_text = _apply_patch_to_lines_with_fallback(target_path, original_text, patch["hunks"])

        with open(abs_path, "w", encoding="utf-8", newline="") as f:
            f.write(updated_text)

    return (True, f"Applied {len(patches)} file patch(es) using Python fallback")

def _apply_patch_to_lines_with_fallback(path: str, original_text: str, hunks: list[dict]) -> str:
    original_lines = original_text.splitlines()
    has_trailing_newline = original_text.endswith("\n")
    out_lines: list[str] = []
    src_idx = 0

    for hunk in hunks:
        expected_old_lines = [h[1:] for h in hunk["lines"] if h and h[0] in (" ", "-")]

        target_idx = max(src_idx, hunk["old_start"] - 1)
        if expected_old_lines and not _matches_subsequence(original_lines, target_idx, expected_old_lines):
            relocated = _find_unique_subsequence(original_lines, src_idx, expected_old_lines)
            if relocated is None:
                raise ValueError(
                    f"Could not uniquely relocate hunk in {path} near line {hunk['old_start']}"
                )
            target_idx = relocated

        if target_idx < src_idx:
            raise ValueError(f"Overlapping hunks in {path}")

        out_lines.extend(original_lines[src_idx:target_idx])
        src_idx = target_idx

        for hline in hunk["lines"]:
            if not hline:
                raise ValueError(f"Malformed empty hunk line in {path}")

            marker = hline[0]
            content = hline[1:]

            if marker == " ":
                if src_idx >= len(original_lines) or original_lines[src_idx] != content:
                    raise ValueError(f"Context mismatch in {path} at source line {src_idx + 1}")
                out_lines.append(content)
                src_idx += 1
            elif marker == "-":
                if src_idx >= len(original_lines) or original_lines[src_idx] != content:
                    raise ValueError(f"Removal mismatch in {path} at source line {src_idx + 1}")
                src_idx += 1
            elif marker == "+":
                out_lines.append(content)
            elif hline.startswith("\\ No newline at end of file"):
                continue
            else:
                raise ValueError(f"Unsupported hunk marker '{marker}' in {path}")

    out_lines.extend(original_lines[src_idx:])
    result_text = "\n".join(out_lines)
    if has_trailing_newline:
        result_text += "\n"
    return result_text

def apply_diff(state: dict) -> dict:
    """
    Parses the last assistant message for a unified diff and applies it.
    Enhanced with detailed diagnostics and logging.
    """
    print("[cody-graph] apply_diff: START", flush=True)
    repo_path = state["repo_path"]
    logs_dir = _ensure_logs_dir(repo_path)
    state["logs_dir"] = logs_dir
    
    messages = state.get("messages", [])
    if not messages:
        error_msg = "No messages found."
        result = {**state, "status": "error", "last_output": error_msg}
        print(f"[cody-graph] apply_diff: ERROR - {error_msg}", flush=True)
        print("[cody-graph] apply_diff: END (error)", flush=True)
        return result

    last_reply = messages[-1]["content"]
    print(f"[cody-graph] [DIAG] LLM response length: {len(last_reply)} chars", flush=True)
    _save_diagnostic(logs_dir, "llm_response_raw", last_reply)

    # Extract the diff block (looking for ```diff ... ``` or raw diff headers)
    print("[cody-graph] [DIAG] Attempting to extract diff from markdown block...", flush=True)
    diff_match = re.search(r"```(?:diff)?\n(.*?)\n```", last_reply, re.DOTALL)
    diff_content = diff_match.group(1) if diff_match else None

    if not diff_content:
        print("[cody-graph] [DIAG] No markdown diff block found, checking for raw diff headers...", flush=True)
        # If no markdown block, try to find raw diff headers
        if "---" in last_reply and "+++" in last_reply:
            diff_content = last_reply
            print("[cody-graph] [DIAG] Found raw diff headers", flush=True)
        else:
            error_msg = "No unified diff found in LLM response."
            print(f"[cody-graph] [DIAG] ERROR - {error_msg}", flush=True)
            _save_diagnostic(logs_dir, "diff_extraction_failed", last_reply)
            result = {**state, "status": "pending", "last_output": error_msg}
            print("[cody-graph] apply_diff: END (no diff)", flush=True)
            return result

    print(f"[cody-graph] [DIAG] Extracted diff content: {len(diff_content)} chars", flush=True)
    
    # Sanitize diff format (handle LLM-generated non-standard formats)
    print("[cody-graph] [DIAG] Sanitizing diff format...", flush=True)
    diff_content = _sanitize_diff(diff_content)
    print(f"[cody-graph] [DIAG] After sanitization: {len(diff_content)} chars", flush=True)

    # Validate UTF-8 encoding in diff content
    print("[cody-graph] [DIAG] Validating UTF-8 encoding in diff...", flush=True)
    is_utf8_valid, utf8_msg = _validate_utf8_in_diff(diff_content)
    if not is_utf8_valid:
        error_msg = f"Diff validation failed: {utf8_msg}"
        print(f"[cody-graph] [DIAG] ERROR - {error_msg}", flush=True)
        _save_diagnostic(logs_dir, "diff_utf8_validation_failed", f"{error_msg}\n\nDiff content:\n{repr(diff_content[:500])}")
        result = {**state, "status": "pending", "last_output": error_msg}
        print("[cody-graph] apply_diff: END (UTF-8 validation failed)", flush=True)
        return result

    # Enforce conservative diff policy for orchestration reliability
    is_safe, policy_msg, diff_summary = _validate_diff_policy(diff_content, state)
    print(f"[cody-graph] [DIAG] Diff policy: {policy_msg}", flush=True)
    if diff_summary:
        print(
            "[cody-graph] [DIAG] Diff summary: "
            f"files={diff_summary.get('file_count')} "
            f"hunks={diff_summary.get('hunk_count')} "
            f"changed_lines={diff_summary.get('changed_lines')}",
            flush=True,
        )

    _save_diagnostic(logs_dir, "diff_extracted", diff_content)
    _save_diagnostic(logs_dir, "diff_policy", f"{policy_msg}\nsummary={diff_summary}")

    if not is_safe:
        # Use the warning signature stored by clippy_agent
        warning_signature = state.get("current_warning_signature")
        attempted = state.get("attempted_warnings", []) or []
        
        if warning_signature and warning_signature not in attempted:
            attempted = attempted + [warning_signature]
            print(f"[cody-graph] [DIAG] Marking warning as attempted: {warning_signature[:80]}", flush=True)
        elif not warning_signature:
            print(f"[cody-graph] [DIAG] No warning signature available to mark as attempted", flush=True)
        
        # Instead of failing, continue to try next warning
        result_state = {
            **state,
            "status": "pending",
            "last_output": f"{policy_msg}\nSkipping this warning and trying next one.",
            "last_command": "apply_diff_rejected",
            "attempted_warnings": attempted,
            "current_warning_signature": None,  # Clear for next attempt
        }
        print(f"[cody-graph] apply_diff: REJECTED - {policy_msg}", flush=True)
        print("[cody-graph] apply_diff: END (rejected, continuing to next warning)", flush=True)
        return result_state

    try:
        # Write the diff to a temporary file
        patch_path = os.path.join(repo_path, "changes.patch")
        print(f"[cody-graph] [DIAG] Writing patch to: {patch_path}", flush=True)
        with open(patch_path, "w") as f:
            f.write(diff_content)

        print("[cody-graph] [DIAG] Looking for patch tool (git or patch)...", flush=True)
        if shutil.which("git"):
            print("[cody-graph] [DIAG] Using 'git apply' with fallback strategies", flush=True)
            result = _run_patch_with_strategies(repo_path)
        elif shutil.which("patch"):
            print("[cody-graph] [DIAG] Using 'patch' command", flush=True)
            result = subprocess.run(
                ["patch", "-p1", "-i", "changes.patch"],
                cwd=repo_path,
                capture_output=True,
                text=True,
            )
        else:
            print("[cody-graph] [DIAG] No git/patch binary found, using Python diff applier fallback", flush=True)
            ok, message = _apply_unified_diff_python(repo_path, diff_content)
            os.remove(patch_path)
            if ok:
                print(f"[cody-graph] apply_diff: SUCCESS - {message}", flush=True)
                result_state = {
                    **state,
                    "status": "pending",
                    "last_output": message,
                    "last_diff": diff_content,
                    "last_command": "apply_diff",
                    "current_warning_signature": None,  # Clear after successful apply
                }
                print("[cody-graph] apply_diff: END (ok)", flush=True)
                return result_state

            error_msg = f"Python diff fallback failed: {message}"
            print(f"[cody-graph] apply_diff: ERROR - {error_msg}", flush=True)
            result_state = {
                **state,
                "status": "error",
                "last_output": error_msg,
            }
            print("[cody-graph] apply_diff: END (error)", flush=True)
            return result_state

        print(f"[cody-graph] [DIAG] Patch tool exit code: {result.returncode}", flush=True)
        if result.stdout:
            print(f"[cody-graph] [DIAG] Patch stdout: {result.stdout[:500]}", flush=True)
            _save_diagnostic(logs_dir, "patch_stdout", result.stdout)
        if result.stderr:
            print(f"[cody-graph] [DIAG] Patch stderr: {result.stderr[:500]}", flush=True)
            _save_diagnostic(logs_dir, "patch_stderr", result.stderr)

        os.remove(patch_path) # Clean up

        if result.returncode == 0:
            success_msg = "Patch applied successfully."
            print(f"[cody-graph] apply_diff: SUCCESS - {success_msg}", flush=True)
            result_state = {
                **state,
                "status": "pending",
                "last_output": success_msg,
                "last_diff": diff_content,
                "last_command": "apply_diff",
                "current_warning_signature": None,  # Clear after successful apply
            }
            print("[cody-graph] apply_diff: END (ok)", flush=True)
            return result_state
        else:
            print("[cody-graph] [DIAG] External patch tool failed; attempting Python fallback", flush=True)
            ok, fallback_msg = _apply_unified_diff_python(repo_path, diff_content)
            if ok:
                success_msg = f"Patch applied via Python fallback after external tool failure: {fallback_msg}"
                print(f"[cody-graph] apply_diff: SUCCESS - {success_msg}", flush=True)
                result_state = {
                    **state,
                    "status": "pending",
                    "last_output": success_msg,
                    "last_diff": diff_content,
                    "last_command": "apply_diff",
                    "current_warning_signature": None,  # Clear after successful apply
                }
                print("[cody-graph] apply_diff: END (ok)", flush=True)
                return result_state

            error_details = (
                f"Patch failed with exit code {result.returncode}: {result.stderr}\n"
                f"Python fallback also failed: {fallback_msg}"
            )
            print(f"[cody-graph] apply_diff: ERROR - {error_details[:200]}", flush=True)
            
            # Mark warning as attempted and continue to next one
            warning_signature = state.get("current_warning_signature")
            attempted = state.get("attempted_warnings", []) or []
            if warning_signature and warning_signature not in attempted:
                attempted = attempted + [warning_signature]
                print(f"[cody-graph] [DIAG] Marking failed patch warning as attempted: {warning_signature[:80]}", flush=True)
            
            result_state = {
                **state,
                "status": "pending",
                "last_output": f"{error_details}\nSkipping this warning and trying next one.",
                "last_command": "apply_diff_rejected",
                "attempted_warnings": attempted,
                "current_warning_signature": None,
            }
            print("[cody-graph] apply_diff: END (patch failed, continuing to next warning)", flush=True)
            return result_state

    except Exception as e:
        error_msg = f"Error applying patch: {e}"
        print(f"[cody-graph] apply_diff: ERROR - {error_msg}", flush=True)
        _save_diagnostic(logs_dir, "patch_exception", str(e))
        
        # Mark warning as attempted and continue to next one
        warning_signature = state.get("current_warning_signature")
        attempted = state.get("attempted_warnings", []) or []
        if warning_signature and warning_signature not in attempted:
            attempted = attempted + [warning_signature]
            print(f"[cody-graph] [DIAG] Marking exception warning as attempted: {warning_signature[:80]}", flush=True)
        
        result_state = {
            **state,
            "status": "pending",
            "last_output": f"{error_msg}\nSkipping this warning and trying next one.",
            "last_command": "apply_diff_rejected",
            "attempted_warnings": attempted,
            "current_warning_signature": None,
        }
        print("[cody-graph] apply_diff: END (exception, continuing to next warning)", flush=True)
        return result_state