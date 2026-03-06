"""
RetryManager: Reusable retry and failure tracking for all phases.

This module provides a unified approach to handling failed fixes across all phases
(clippy, refactoring, performance, ELO gain, etc). It tracks:
- Failed patches/warnings to avoid retrying them
- Repair attempt counts per issue
- Rollback strategy and next-step routing
"""

import json
import os
from datetime import datetime
from pathlib import Path
from typing import Optional, Tuple

from state.cody_state import CodyState


class RetryManager:
    """Manages retries, repair attempts, and state tracking across phases."""
    
    MAX_REPAIR_ATTEMPTS = 2  # Allow up to 2 LLM repair attempts per patch
    
    def __init__(self, repo_path: str):
        self.repo_path = repo_path
        self.logs_dir = os.path.join(repo_path, ".cody_logs")
        os.makedirs(self.logs_dir, exist_ok=True)
    
    def get_failed_warnings_file(self, phase: str) -> str:
        """Get the path to the failed warnings tracking file for a phase."""
        return os.path.join(self.logs_dir, f"{phase}_failed_warnings.json")
    
    def load_failed_warnings(self, phase: str) -> dict:
        """Load the set of failed warning signatures for a phase.
        
        Returns: {
            "signature1": {"failed_at": "2024-...", "attempts": 2, "reason": "..."},
            ...
        }
        """
        filepath = self.get_failed_warnings_file(phase)
        if not os.path.exists(filepath):
            return {}
        try:
            with open(filepath, "r") as f:
                return json.load(f)
        except Exception as e:
            print(f"[cody-graph] [DIAG] Error loading failed warnings: {e}", flush=True)
            return {}
    
    def save_failed_warnings(self, phase: str, failed_warnings: dict) -> None:
        """Save the set of failed warning signatures for a phase."""
        filepath = self.get_failed_warnings_file(phase)
        try:
            with open(filepath, "w") as f:
                json.dump(failed_warnings, f, indent=2)
            print(f"[cody-graph] [DIAG] Saved failed warnings to {filepath}", flush=True)
        except Exception as e:
            print(f"[cody-graph] [DIAG] Error saving failed warnings: {e}", flush=True)
    
    def mark_warning_failed(
        self, 
        phase: str, 
        warning_signature: Optional[str],
        reason: str,
        attempt_count: int = 1
    ) -> None:
        """Mark a warning/issue as failed so it won't be retried.
        
        Args:
            phase: Current phase (e.g., "clippy")
            warning_signature: Unique signature of the warning/issue
            reason: Why it failed (e.g., "build error after repair attempt", "too many hunks")
            attempt_count: How many repair attempts were made
        """
        if not warning_signature:
            print("[cody-graph] [DIAG] Warning signature is None, cannot mark as failed", flush=True)
            return
        
        failed = self.load_failed_warnings(phase)
        failed[warning_signature] = {
            "failed_at": datetime.now().isoformat(),
            "attempts": attempt_count,
            "reason": reason,
        }
        self.save_failed_warnings(phase, failed)
        print(
            f"[cody-graph] [DIAG] Marked warning as failed: {warning_signature} "
            f"({attempt_count} attempts, reason: {reason})",
            flush=True,
        )
    
    def is_warning_failed(self, phase: str, warning_signature: Optional[str]) -> bool:
        """Check if a warning has already failed in this phase."""
        if not warning_signature:
            return False
        failed = self.load_failed_warnings(phase)
        return warning_signature in failed
    
    def get_repair_attempt_count(self, state: CodyState) -> int:
        """Get the number of repair attempts made for the current issue.
        
        Tracked via the state's "repair_attempts" counter (incremented per repair).
        """
        return int(state.get("repair_attempts", 0) or 0)
    
    def should_attempt_repair(self, state: CodyState) -> bool:
        """Determine if we should attempt LLM repair for a build failure.
        
        Returns True if:
        - This is the first or second repair attempt
        - We have a patch that was just applied
        - We have a current warning signature to track
        """
        attempts = self.get_repair_attempt_count(state)
        has_diff = bool(state.get("last_diff"))
        has_signature = bool(state.get("current_warning_signature"))
        
        result = (attempts < self.MAX_REPAIR_ATTEMPTS) and has_diff and has_signature
        print(
            f"[cody-graph] [DIAG] Repair attempt check: "
            f"attempts={attempts}/{self.MAX_REPAIR_ATTEMPTS}, "
            f"has_diff={has_diff}, has_signature={has_signature}, "
            f"should_repair={result}",
            flush=True,
        )
        return result
    
    def increment_repair_attempts(self, state: CodyState) -> CodyState:
        """Increment the repair attempt counter in state."""
        current = int(state.get("repair_attempts", 0) or 0)
        return {
            **state,
            "repair_attempts": current + 1,
        }
    
    def reset_repair_attempts(self, state: CodyState) -> CodyState:
        """Reset repair attempts counter (for new patches)."""
        return {
            **state,
            "repair_attempts": 0,
        }
    
    def log_repair_failure(
        self,
        phase: str,
        warning_signature: Optional[str],
        build_error: str,
        repair_attempts: int,
    ) -> None:
        """Create a detailed log of why a repair attempt failed."""
        if not warning_signature:
            return
        
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        log_file = os.path.join(self.logs_dir, f"{timestamp}_repair_failure.log")
        
        content = f"""REPAIR FAILURE LOG
==================

Phase: {phase}
Warning Signature: {warning_signature}
Repair Attempts: {repair_attempts}
Timestamp: {timestamp}

BUILD ERROR THAT REQUIRED REPAIR:
{build_error}
"""
        
        try:
            with open(log_file, "w") as f:
                f.write(content)
            print(f"[cody-graph] [DIAG] Logged repair failure to {log_file}", flush=True)
        except Exception as e:
            print(f"[cody-graph] [DIAG] Error logging repair failure: {e}", flush=True)


def create_retry_manager(state: CodyState) -> RetryManager:
    """Factory function to create a RetryManager from state."""
    return RetryManager(state.get("repo_path", ""))
