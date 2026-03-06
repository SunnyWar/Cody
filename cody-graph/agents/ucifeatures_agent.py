import json
import os
import re
import subprocess
import difflib
from datetime import datetime
from pathlib import Path
from typing import Any

from openai import OpenAI

from state.cody_state import CodyState


MAX_SNIPPET_LINES = 220
MAX_EXTRA_FILES = 4


def _load_config(repo_path: str) -> dict:
    config_override = os.environ.get("CODY_CONFIG_PATH")
    if config_override:
        config_path = Path(config_override)
    else:
        config_path = Path(repo_path) / "cody-agent" / "config.json"
    if not config_path.exists():
        return {}
    try:
        return json.loads(config_path.read_text(encoding="utf-8"))
    except Exception:
        return {}


def _select_model(config: dict, phase: str) -> str:
    models = config.get("models", {}) if isinstance(config, dict) else {}
    return models.get(phase) or models.get("clippy") or config.get("model") or "gpt-4o-mini"


def _read_file_head(repo_path: str, rel_path: str, max_lines: int = MAX_SNIPPET_LINES) -> str:
    full = Path(repo_path) / rel_path
    if not full.exists() or not full.is_file():
        return ""
    try:
        lines = full.read_text(encoding="utf-8").splitlines()
    except Exception:
        return ""

    shown = lines[:max_lines]
    numbered = [f"{i + 1:4d} | {line}" for i, line in enumerate(shown)]
    return f"File: {rel_path}\n" + "\n".join(numbered)


def _read_file_raw_head(repo_path: str, rel_path: str, max_lines: int = MAX_SNIPPET_LINES) -> str:
    full = Path(repo_path) / rel_path
    if not full.exists() or not full.is_file():
        return ""
    try:
        lines = full.read_text(encoding="utf-8").splitlines()
    except Exception:
        return ""

    shown = "\n".join(lines[:max_lines])
    return f"File: {rel_path}\n```rust\n{shown}\n```"


def _extract_command_patterns(uciapi_text: str) -> list[str]:
    patterns: list[str] = []
    for match in re.finditer(r'\"([^\"]+)\"\s*=>', uciapi_text):
        cmd = match.group(1).strip()
        if cmd:
            patterns.append(cmd)
    for match in re.finditer(r'cmd\s+if\s+cmd\.starts_with\(\"([^\"]+)\"\)', uciapi_text):
        cmd = match.group(1).strip()
        if cmd:
            patterns.append(f"{cmd}*")

    seen = set()
    ordered = []
    for p in patterns:
        if p not in seen:
            seen.add(p)
            ordered.append(p)
    return ordered


def _safe_json_extract(text: str) -> dict[str, Any] | None:
    try:
        return json.loads(text)
    except Exception:
        pass

    start = text.find("{")
    end = text.rfind("}")
    if start >= 0 and end > start:
        candidate = text[start : end + 1]
        try:
            return json.loads(candidate)
        except Exception:
            return None
    return None


def _rg_search_files(repo_path: str, terms: list[str], max_files: int = MAX_EXTRA_FILES) -> list[str]:
    files: list[str] = []
    seen = set()

    for term in terms:
        if len(files) >= max_files:
            break
        try:
            result = subprocess.run(
                ["rg", "-n", term, "engine/src", "bitboard/src"],
                cwd=repo_path,
                capture_output=True,
                text=True,
                check=False,
            )
        except Exception:
            continue

        for line in result.stdout.splitlines():
            path = line.split(":", 1)[0].strip().replace("\\", "/")
            if not path.endswith(".rs"):
                continue
            if path not in seen:
                seen.add(path)
                files.append(path)
                if len(files) >= max_files:
                    break

    return files


def _safe_rel_path(path: str) -> str:
    rel = path.strip().replace("\\", "/")
    while rel.startswith("./"):
        rel = rel[2:]
    return rel


def _extract_target_files_from_edits(edits: list[dict[str, Any]], max_files: int = 3) -> list[str]:
    files: list[str] = []
    seen = set()
    for edit in edits:
        rel = _safe_rel_path(str(edit.get("file") or ""))
        if rel and rel not in seen:
            seen.add(rel)
            files.append(rel)
            if len(files) >= max_files:
                break
    return files


def _build_diff_from_structured_edits(repo_path: str, edits: list[dict[str, Any]]) -> tuple[bool, str, str]:
    """Apply structured find/replace edits in-memory and return a unified diff.

    Returns: (ok, message, diff_content)
    """
    if not edits:
        return (False, "No edits provided.", "")

    originals: dict[str, str] = {}
    updated: dict[str, str] = {}

    for edit in edits:
        rel = _safe_rel_path(str(edit.get("file") or ""))
        find = str(edit.get("find") or "")
        replace = str(edit.get("replace") or "")

        if not rel or not find:
            return (False, "Each edit must include non-empty 'file' and 'find'.", "")
        if rel.startswith("/") or ".." in rel.split("/"):
            return (False, f"Invalid file path in edit: {rel}", "")

        abs_path = Path(repo_path) / rel
        if not abs_path.exists() or not abs_path.is_file():
            return (False, f"Edit target not found: {rel}", "")

        if rel not in originals:
            try:
                originals[rel] = abs_path.read_text(encoding="utf-8")
            except Exception as e:
                return (False, f"Failed to read {rel}: {e}", "")
            updated[rel] = originals[rel]

        occurrence_count = updated[rel].count(find)
        if occurrence_count != 1:
            return (
                False,
                f"Edit find-block for {rel} matched {occurrence_count} times; expected exactly 1.",
                "",
            )

        updated[rel] = updated[rel].replace(find, replace, 1)

    changed_paths = [p for p in updated.keys() if updated[p] != originals[p]]
    if not changed_paths:
        return (False, "Structured edits produced no effective changes.", "")

    diff_chunks: list[str] = []
    for rel in sorted(changed_paths):
        before_lines = originals[rel].splitlines()
        after_lines = updated[rel].splitlines()
        chunk = list(
            difflib.unified_diff(
                before_lines,
                after_lines,
                fromfile=f"a/{rel}",
                tofile=f"b/{rel}",
                lineterm="",
            )
        )
        if chunk:
            diff_chunks.extend(chunk)

    if not diff_chunks:
        return (False, "Failed to build unified diff from structured edits.", "")

    return (True, "Structured edits converted to unified diff.", "\n".join(diff_chunks) + "\n")


def ucifeatures_agent(state: CodyState) -> CodyState:
    print("[cody-graph] ucifeatures_agent: START", flush=True)

    api_key = os.environ.get("OPENAI_API_KEY")
    if not api_key:
        return {
            **state,
            "last_command": "ucifeatures_agent",
            "last_output": "Missing OPENAI_API_KEY environment variable.",
            "status": "error",
        }

    repo_path = state.get("repo_path", "")
    config = _load_config(repo_path)
    model = _select_model(config, "UCIfeatures")
    print(f"[cody-graph] [DIAG] Using model: {model} for phase 'UCIfeatures'", flush=True)

    main_rel = "engine/src/main.rs"
    uci_rel = "engine/src/api/uciapi.rs"

    main_snippet = _read_file_head(repo_path, main_rel, max_lines=180)
    uci_snippet = _read_file_head(repo_path, uci_rel, max_lines=260)
    main_raw = _read_file_raw_head(repo_path, main_rel, max_lines=220)
    uci_raw = _read_file_raw_head(repo_path, uci_rel, max_lines=320)

    if not main_snippet and not uci_snippet:
        return {
            **state,
            "last_command": "ucifeatures_agent",
            "last_output": "Could not read main.rs / uciapi.rs context for UCIfeatures phase.",
            "status": "error",
        }

    command_patterns = _extract_command_patterns(uci_snippet)
    commands_text = "\n".join(f"- {c}" for c in command_patterns) if command_patterns else "- (none parsed)"

    client = OpenAI(api_key=api_key)

    recommend_system = (
        "You are Cody's UCI feature planning specialist for a Rust chess engine. "
        "Analyze current UCI command handling and identify one missing or incomplete tournament-relevant command "
        "to implement in a single focused change."
    )

    recommend_user = (
        "Examine engine/src/main.rs and engine/src/api/uciapi.rs.\n\n"
        "CURRENTLY HANDLED COMMAND PATTERNS (parsed):\n"
        f"{commands_text}\n\n"
        "Return STRICT JSON only with this schema:\n"
        "{\n"
        "  \"recommended_command\": \"string\",\n"
        "  \"reason\": \"short string\",\n"
        "  \"search_terms\": [\"term1\", \"term2\"],\n"
        "  \"likely_files\": [\"engine/src/...\"]\n"
        "}\n\n"
        "Selection criteria:\n"
        "- High tournament impact, low implementation risk\n"
        "- Can be completed in one coherent patch\n"
        "- Avoid huge refactors\n\n"
        "MAIN_RS:\n"
        f"{main_snippet}\n\n"
        "UCIAPI_RS:\n"
        f"{uci_snippet}"
    )

    try:
        print("[cody-graph] [DIAG] UCIfeatures call 1/2: recommendation", flush=True)
        rec_resp = client.chat.completions.create(
            model=model,
            messages=[
                {"role": "system", "content": recommend_system},
                {"role": "user", "content": recommend_user},
            ],
        )
        recommendation_text = rec_resp.choices[0].message.content or ""
    except Exception as e:
        return {
            **state,
            "last_command": "ucifeatures_agent",
            "last_output": f"UCIfeatures recommendation call failed: {e}",
            "status": "error",
        }

    recommendation_json = _safe_json_extract(recommendation_text) or {}
    recommended_command = str(recommendation_json.get("recommended_command") or "").strip()
    reason = str(recommendation_json.get("reason") or "").strip()
    search_terms_raw = recommendation_json.get("search_terms") or []
    likely_files_raw = recommendation_json.get("likely_files") or []

    search_terms = [str(t).strip() for t in search_terms_raw if str(t).strip()]
    likely_files = [str(f).strip().replace("\\", "/") for f in likely_files_raw if str(f).strip()]

    if recommended_command:
        print(f"[cody-graph] [DIAG] Recommended UCI command: {recommended_command}", flush=True)
    else:
        print("[cody-graph] [DIAG] Recommendation JSON missing command; continuing with fallback context", flush=True)

    # Search pass after recommendation to gather supporting code context.
    search_basis = search_terms[:]
    if recommended_command:
        search_basis.extend([recommended_command, f"{recommended_command} ", f"{recommended_command}\""])
    extra_files = _rg_search_files(repo_path, search_basis, max_files=MAX_EXTRA_FILES)

    merged_files = []
    seen_files = set()
    for path in likely_files + extra_files:
        if path in (main_rel, uci_rel):
            continue
        if path.endswith(".rs") and path not in seen_files:
            seen_files.add(path)
            merged_files.append(path)
        if len(merged_files) >= MAX_EXTRA_FILES:
            break

    extra_context_blocks = []
    extra_raw_blocks = []
    for rel in merged_files:
        block = _read_file_head(repo_path, rel, max_lines=180)
        if block:
            extra_context_blocks.append(block)
        raw_block = _read_file_raw_head(repo_path, rel, max_lines=220)
        if raw_block:
            extra_raw_blocks.append(raw_block)

    impl_system = (
        "You are Cody's UCI implementation engineer. "
        "Implement exactly ONE recommended UCI feature change in Rust. "
        "You may modify multiple files if required. "
        "Return structured JSON edits only."
    )

    recommendation_summary = recommendation_text
    if recommendation_json:
        recommendation_summary = json.dumps(recommendation_json, indent=2)

    impl_user_parts = [
        "Implement the recommended UCI command change from the recommendation below.",
        "Recommendation:",
        recommendation_summary,
        "Output format (STRICT JSON only, no markdown):",
        "{",
        "  \"summary\": \"short text\",",
        "  \"edits\": [",
        "    { \"file\": \"engine/src/...rs\", \"find\": \"exact existing text\", \"replace\": \"new text\" }",
        "  ]",
        "  \"notes\": \"optional\"",
        "}",
        "Rules:",
        "- Preserve existing behavior outside this feature.",
        "- Keep implementation focused and minimal.",
        "- 'find' must be exact, copied from provided context.",
        "- Each edit must target exactly one occurrence.",
        "- You may include multiple edits/files when necessary.",
        "- If not feasible, return JSON with an empty edits array and explain in notes.",
        "Core context:",
        main_raw or main_snippet,
        uci_raw or uci_snippet,
    ]

    if extra_raw_blocks:
        impl_user_parts.append("Additional searched context:")
        impl_user_parts.append("\n\n".join(extra_raw_blocks))
    elif extra_context_blocks:
        impl_user_parts.append("Additional searched context:")
        impl_user_parts.append("\n\n".join(extra_context_blocks))

    impl_user = "\n\n".join(impl_user_parts)

    try:
        print("[cody-graph] [DIAG] UCIfeatures call 2/2: implementation", flush=True)
        impl_resp = client.chat.completions.create(
            model=model,
            messages=[
                {"role": "system", "content": impl_system},
                {"role": "user", "content": impl_user},
            ],
        )
        impl_raw = impl_resp.choices[0].message.content or ""
    except Exception as e:
        return {
            **state,
            "last_command": "ucifeatures_agent",
            "last_output": f"UCIfeatures implementation call failed: {e}",
            "status": "error",
        }

    impl_json = _safe_json_extract(impl_raw)
    repair_raw = ""
    if not impl_json:
        reply = "UCIfeatures implementation failed: implementation output was not valid JSON."
    else:
        edits = impl_json.get("edits") or []
        if not isinstance(edits, list):
            edits = []

        ok_diff, diff_msg, diff_content = _build_diff_from_structured_edits(repo_path, edits)
        if not ok_diff and edits:
            # One recovery attempt: provide exact current file content and ask for corrected edits.
            target_files = _extract_target_files_from_edits(edits, max_files=3)
            target_blocks = [
                _read_file_raw_head(repo_path, rel, max_lines=420)
                for rel in target_files
            ]
            target_blocks = [b for b in target_blocks if b]

            repair_user = "\n\n".join([
                "The previous structured edits failed to apply.",
                f"Failure reason: {diff_msg}",
                "Return corrected STRICT JSON only with the same schema {summary, edits, notes}.",
                "Rules:",
                "- Use exact find text copied from the CURRENT file content below.",
                "- Keep one focused UCI feature implementation.",
                "- If not feasible, return empty edits and explain in notes.",
                "Previous JSON:",
                json.dumps(impl_json, indent=2),
                "Current target file content:",
                "\n\n".join(target_blocks) if target_blocks else (uci_raw or uci_snippet),
            ])

            try:
                print("[cody-graph] [DIAG] UCIfeatures call 3/3: edit repair", flush=True)
                repair_resp = client.chat.completions.create(
                    model=model,
                    messages=[
                        {"role": "system", "content": impl_system},
                        {"role": "user", "content": repair_user},
                    ],
                )
                repair_raw = repair_resp.choices[0].message.content or ""
                repair_json = _safe_json_extract(repair_raw)
                if repair_json:
                    repair_edits = repair_json.get("edits") or []
                    if isinstance(repair_edits, list):
                        ok_diff, diff_msg, diff_content = _build_diff_from_structured_edits(repo_path, repair_edits)
                        if ok_diff:
                            impl_json = repair_json
            except Exception as e:
                print(f"[cody-graph] [DIAG] UCIfeatures repair call failed: {e}", flush=True)

        if ok_diff:
            reply = f"```diff\n{diff_content}```"
        else:
            summary = str(impl_json.get("summary") or "").strip()
            notes = str(impl_json.get("notes") or "").strip()
            explanation = summary or notes or "No feasible UCI feature edit produced."
            reply = f"{explanation}\n\n{diff_msg}"

    logs_dir = state.get("logs_dir") or os.path.join(repo_path, ".cody_logs")
    os.makedirs(logs_dir, exist_ok=True)
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    log_path = os.path.join(logs_dir, f"{timestamp}_ucifeatures_trace.txt")
    try:
        with open(log_path, "w", encoding="utf-8") as f:
            f.write("=== RECOMMENDATION RESPONSE ===\n")
            f.write(recommendation_text + "\n\n")
            f.write("=== IMPLEMENTATION RAW RESPONSE ===\n")
            f.write(impl_raw + "\n\n")
            if repair_raw:
                f.write("=== IMPLEMENTATION REPAIR RAW RESPONSE ===\n")
                f.write(repair_raw + "\n\n")
            f.write("=== IMPLEMENTATION RESPONSE ===\n")
            f.write(reply)
    except Exception:
        pass

    # Ensure apply_diff reads this response as the latest assistant message.
    new_messages = state.get("messages", []) + [{"role": "assistant", "content": reply}]

    result = {
        **state,
        "messages": new_messages,
        "llm_response": reply,
        "last_command": "ucifeatures_agent",
        "last_output": (
            f"UCIfeatures recommendation: {recommended_command or '(unknown)'}"
            + (f" - {reason}" if reason else "")
        ),
        "status": "pending",
        "logs_dir": logs_dir,
        "ucifeatures_recommendation": recommendation_json or {"raw": recommendation_text},
    }
    print(f"[cody-graph] [DIAG] UCIfeatures implementation contains diff block: {'```diff' in reply}", flush=True)
    print("[cody-graph] ucifeatures_agent: END", flush=True)
    return result
