"""
Master Orchestrator for Cody AI Improvement Workflow

Coordinates the improvement workflow with one merge per invocation:
1. Each run performs exactly ONE code improvement and exits
2. State tracks current phase (refactoring → performance → features)
3. Scheduled tasks can be run multiple times daily, each producing one change

IMPORTANT: Only real code changes count (.rs files, data files). TODO list changes
do not count as code changes. Tasks that only modify TODO files are skipped, and 
the orchestrator moves to the next task in sequence.

Workflow progression:
- Refactoring phase: analyze once, then execute tasks one per run (only code changes)
- Clippy phase: run after each phase to keep warnings in check
- Performance phase: analyze once, then execute tasks one per run (only code changes)
- Clippy phase: run after each phase to keep warnings in check
- Features phase: execute up to 3 features (one per run), rerun earlier phases if large diffs
- Clippy phase: run after each phase to keep warnings in check

State file tracks: current_phase, phase_started (datetime), features_completed
"""

import os
import sys
import json
import subprocess
from pathlib import Path
from datetime import datetime
from typing import Optional, Dict, Any

# Import our modules
from todo_manager import TodoList
import refactoring_analyzer
import refactoring_executor
import performance_analyzer
import performance_executor
import clippy_analyzer
import clippy_executor
import features_analyzer
import features_executor


class Orchestrator:
    """Master orchestrator for single-merge-per-run workflow."""
    
    STATE_FILE = "orchestrator_state.json"
    MAIN_PHASES = ["refactoring", "performance", "features"]
    
    def __init__(self, repo_root: Path, config: dict):
        self.repo_root = repo_root
        self.config = config
        self.log_file = repo_root / "orchestrator.log"
        self.state_file = repo_root / self.STATE_FILE
        self.state = self._load_state()
        
    def log(self, message: str):
        """Log a message to both console and file."""
        if not message:
            return

        content = " ".join(message.splitlines()).strip()
        if not content:
            return

        timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        log_line = f"[{timestamp}] {content}"
        print(log_line)
        
        with open(self.log_file, 'a', encoding='utf-8') as f:
            f.write(log_line + "\n")
    
    def _load_state(self) -> Dict[str, Any]:
        """Load orchestration state, Initialize if missing."""
        if self.state_file.exists():
            with open(self.state_file) as f:
                return json.load(f)
        
        # Initialize new state
        return {
            "current_phase": "refactoring",      # which phase we're in
            "phase_started": datetime.now().isoformat(),  # when we started this phase
            "analysis_done": False,              # have we analyzed current phase?
            "features_completed": 0,             # count of features done (for 3-feature limit)
            "resume_phase": None,                # where to resume after clippy
            "last_run": None,
            "run_count": 0
        }
    
    def _save_state(self):
        """Persist state to file."""
        self.state["last_run"] = datetime.now().isoformat()
        with open(self.state_file, 'w') as f:
            json.dump(self.state, f, indent=2)
    
    def _advance_main_phase(self, from_phase: str):
        """Move to the next main phase, inserting clippy cleanup in between."""
        current_idx = self.MAIN_PHASES.index(from_phase)
        if current_idx < len(self.MAIN_PHASES) - 1:
            next_phase = self.MAIN_PHASES[current_idx + 1]
        else:
            next_phase = None

        self._set_clippy_phase(next_phase)

    def _set_clippy_phase(self, resume_phase: Optional[str]):
        """Switch to clippy phase and store where to resume afterward."""
        self.state["current_phase"] = "clippy"
        self.state["resume_phase"] = resume_phase
        self.state["analysis_done"] = False
        self.state["phase_started"] = datetime.now().isoformat()
        self._save_state()
    
    def _validate_and_exit_if_ready(self):
        """Validate changes using validate_cargo.py and exit if ready to commit."""
        self.log("\n🔍 Validating changes with validate_cargo.py...")
        try:
            result = subprocess.run(
                ["python", "cody-agent/validate_cargo.py"],
                cwd=self.repo_root,
                capture_output=True,
                text=True,
                check=False
            )

            if result.returncode == 0:
                self.log("\n✅ Validation passed. Changes are ready to be committed.")
                sys.exit(0)
            else:
                self.log("\n❌ Validation failed. Please fix the issues below:")
                self.log(result.stdout)
                self.log(result.stderr)
                sys.exit(1)
        except Exception as e:
            self.log(f"\n⚠️ Validation failed due to an error: {e}")
            sys.exit(1)

    def _has_code_changes(self) -> bool:
        """Check if git diff contains actual code changes (not just TODO files).
        If .rs files are changed, validate and exit.
        """
        try:
            # Get list of modified files
            result = subprocess.run(
                ["git", "diff", "--name-only", "--cached"],
                cwd=self.repo_root,
                capture_output=True,
                text=True,
                check=False
            )

            if result.returncode != 0:
                return False

            changed_files = result.stdout.strip().split('\n')

            # Check if any non-TODO files were changed
            for file in changed_files:
                if not file:
                    continue
                # Include .rs files and any non-todo data files
                if file.endswith('.rs'):
                    self._validate_and_exit_if_ready()
                if file.endswith('.json') and 'todo' not in file.lower():
                    return True
                if file.endswith('.toml') and 'todo' not in file.lower():
                    return True
                if file.endswith('.yaml') and 'todo' not in file.lower():
                    return True
                if file.endswith('.yml') and 'todo' not in file.lower():
                    return True
                # Data files in specific directories
                if '.rs' in file or ('data' in file and 'todo' not in file.lower()):
                    return True

            return False
        except Exception as e:
            self.log(f"⚠️ Could not check for code changes: {e}")
            return False
    
    def run_single_improvement(self) -> bool:
        """Execute a single improvement task and exit.
        
        Returns True if an improvement was made, False if workflow is complete.
        """
        self.state["run_count"] = self.state.get("run_count", 0) + 1
        self._save_state()
        self.log(f"ORCHESTRATOR RUN #{self.state['run_count']}")
        self.log(f"Phase: {self.state['current_phase'].upper()}")
        
        try:
            # Ensure we're on main and clean
            self.log("🔧 Preparing workspace...")
            subprocess.run(["git", "checkout", "main"], cwd=self.repo_root, check=False)
            
            phase = self.state["current_phase"]
            
            if phase == "refactoring":
                return self._run_refactoring_task()
            elif phase == "performance":
                return self._run_performance_task()
            elif phase == "clippy":
                return self._run_clippy_task()
            elif phase == "features":
                return self._run_features_task()
            else:
                self.log("✅ WORKFLOW COMPLETE!")
                self.log(f"   All phases finished. Run count: {self.state['run_count']}")
                self._save_state()
                return False
                
        except KeyboardInterrupt:
            self.log("⚠️ Run interrupted by user")
            sys.exit(1)
        except Exception as e:
            self.log(f"❌ Run failed with error: {e}")
            import traceback
            self.log(traceback.format_exc())
            sys.exit(1)
    
    def _run_refactoring_task(self) -> bool:
        """Run one refactoring task. Return True if actual code was merged, False if phase changed."""
        self.log("REFACTORING PHASE")
        
        # Analyze if not yet done
        if not self.state["analysis_done"]:
            self.log("Analyzing refactoring opportunities...")
            added = refactoring_analyzer.analyze(self.repo_root, self.config)
            self.log(f"Found {added} refactoring opportunities")
            self.state["analysis_done"] = True
            self._save_state()
        
        # Get next task
        todo_list = TodoList("refactoring", self.repo_root)
        next_item = todo_list.get_next_item()
        
        if not next_item:
            self.log("✅ Refactoring phase complete, moving to performance...")
            self._advance_main_phase("refactoring")
            # Recursively execute next phase
            return self.run_single_improvement()
        
        # Execute the single task
        self.log(f"📝 Working on: {next_item.id} - {next_item.title}")
        success = refactoring_executor.execute_refactoring(
            next_item.id,
            self.repo_root,
            self.config
        )
        
        if success:
            # Check for actual code changes (exclude TODO files)
            if self._has_code_changes():
                self.log(f"✅ Successfully completed: {next_item.id}")
                self._create_checkpoint(f"Refactoring: {next_item.id}")
                self._save_state()
                return True
            else:
                self.log(f"ℹ️ Task completed but no code changes made (TODO-only): {next_item.id}")
                # Mark as done in TODO but continue to next task
                return self._run_refactoring_task()
        else:
            self.log(f"❌ Failed: {next_item.id}")
            self._save_state()
            return False
    
    def _run_performance_task(self) -> bool:
        """Run one performance task. Return True if actual code was merged, False if phase changed."""
        self.log("PERFORMANCE OPTIMIZATION PHASE")
        
        # Analyze if not yet done
        if not self.state["analysis_done"]:
            self.log("Analyzing performance opportunities...")
            added = performance_analyzer.analyze(self.repo_root, self.config)
            self.log(f"Found {added} performance opportunities")
            self.state["analysis_done"] = True
            self._save_state()
        
        # Get next task
        todo_list = TodoList("performance", self.repo_root)
        next_item = todo_list.get_next_item()
        
        if not next_item:
            self.log("✅ Performance phase complete, moving to features...")
            self._advance_main_phase("performance")
            # Recursively execute next phase
            return self.run_single_improvement()
        
        # Execute the single task
        self.log(f"📝 Working on: {next_item.id} - {next_item.title}")
        success = performance_executor.execute_optimization(
            next_item.id,
            self.repo_root,
            self.config
        )
        
        if success:
            # Check for actual code changes (exclude TODO files)
            if self._has_code_changes():
                self.log(f"✅ Successfully completed: {next_item.id}")
                self._create_checkpoint(f"Performance: {next_item.id}")
                self._save_state()
                return True
            else:
                self.log(f"ℹ️ Task completed but no code changes made (TODO-only): {next_item.id}")
                # Mark as done in TODO but continue to next task
                return self._run_performance_task()
        else:
            self.log(f"❌ Failed: {next_item.id}")
            self._save_state()
            return False
    
    def _run_features_task(self) -> bool:
        """Run one feature task. Return True if actual code was merged, False if workflow done."""
        self.log("WORLD-CLASS FEATURES PHASE")
        
        features_done = self.state.get("features_completed", 0)
        max_features = 3
        
        # Check if we've hit feature limit
        if features_done >= max_features:
            self.log(f"✅ Features phase complete (executed {features_done}/{max_features})")
            self._advance_main_phase("features")
            return self.run_single_improvement()

        # Analyze if not yet done
        if not self.state["analysis_done"]:
            self.log("Analyzing feature opportunities...")
            added = features_analyzer.analyze(self.repo_root, self.config)
            self.log(f"Found {added} feature opportunities")
            self.state["analysis_done"] = True
            self._save_state()

        # Get next task
        todo_list = TodoList("features", self.repo_root)
        next_item = todo_list.get_next_item()

        if not next_item:
            self.log(f"✅ No more features available ({features_done}/{max_features})")
            self._advance_main_phase("features")
            return self.run_single_improvement()

        # Execute the single feature
        self.log(f"📝 Feature {features_done + 1}/{max_features}: {next_item.id} - {next_item.title}")
        success, diff_size = features_executor.execute_feature(
            next_item.id,
            self.repo_root,
            self.config
        )

        if success:
            # Check for actual code changes (exclude TODO files)
            if self._has_code_changes():
                self.log(f"✅ Successfully completed: {next_item.id}")
                self._create_checkpoint(f"Feature: {next_item.id}")
                self.state["features_completed"] = features_done + 1

                # If large diff, trigger refactoring and performance passes
                if diff_size == "large":
                    self.log("📏 Large diff detected - queueing refactoring & performance passes")
                    self.log("   Reset state to refactoring phase for quality improvements")
                    self.state["current_phase"] = "refactoring"
                    self.state["analysis_done"] = False
                    self.state["phase_started"] = datetime.now().isoformat()
                    self.state["resume_phase"] = None

                self._save_state()
                return True
            else:
                self.log(f"ℹ️ Task completed but no code changes made (TODO-only): {next_item.id}")
                # Continue to next feature
                return self._run_features_task()
        else:
            self.log(f"❌ Failed: {next_item.id}")
            self._save_state()
            return False

    def _run_clippy_task(self) -> bool:
        """Run one clippy task. Return True if actual code was merged, False if phase changed."""
        self.log("CLIPPY WARNINGS PHASE")

        # Analyze if not yet done
        if not self.state["analysis_done"]:
            self.log("Analyzing clippy warnings...")
            added = clippy_analyzer.analyze(self.repo_root, self.config)
            self.log(f"Found {added} clippy warnings")
            self.state["analysis_done"] = True
            self.state["clippy_reanalyzed"] = False  # Track if we've re-analyzed this cycle
            self._save_state()

        # Get next task
        todo_list = TodoList("clippy", self.repo_root)
        
        # Reset any stuck in-progress items from crashed runs
        reset_count = todo_list.reset_in_progress()
        if reset_count > 0:
            self.log(f"🔄 Reset {reset_count} stuck in-progress item(s) from previous run")
        
        next_item = todo_list.get_next_item()

        if not next_item:
            # Check if we have any not-started items at all
            not_started_count = todo_list.count_by_status("not-started")
            reanalyzed = self.state.get("clippy_reanalyzed", False)
            
            if not_started_count == 0 and not reanalyzed:
                # All items are completed/failed - re-analyze ONCE to find new warnings
                self.log("All items processed. Re-analyzing for new clippy warnings...")
                self.state["analysis_done"] = False
                self.state["clippy_reanalyzed"] = True
                self._save_state()
                return self._run_clippy_task()  # Retry with fresh analysis
            
            self.log("✅ Clippy phase complete, resuming next phase...")
            return self._resume_phase_after_clippy()

        # Execute the single task
        self.log(f"📝 Working on: {next_item.id} - {next_item.title}")
        success = clippy_executor.execute_clippy_fix(
            next_item.id,
            self.repo_root,
            self.config
        )

        if success:
            # Check for actual code changes (exclude TODO files)
            if self._has_code_changes():
                self.log(f"✅ Successfully completed: {next_item.id}")
                self._create_checkpoint(f"Clippy: {next_item.id}")
                self._save_state()
                return True
            else:
                self.log(f"ℹ️ Task completed but no code changes made (TODO-only): {next_item.id}")
                # Mark as done in TODO but continue to next task
                return self._run_clippy_task()
        else:
            self.log(f"❌ Failed: {next_item.id}")
            self._save_state()
            return False

    def _resume_phase_after_clippy(self):
        """Resume the intended phase after completing clippy."""
        resume_phase = self.state.get("resume_phase")
        if resume_phase:
            self.log(f"Resuming phase: {resume_phase.upper()} after clippy.")
            self.state["current_phase"] = resume_phase
            self.state["resume_phase"] = None
            self.state["analysis_done"] = False
            self.state["phase_started"] = datetime.now().isoformat()
            self._save_state()
            return self.run_single_improvement()
        else:
            self.log("✅ Clippy phase complete. No phase to resume.")
            self.log("   Workflow complete - all tasks finished.")
            return False

    def _create_checkpoint(self, name: str):
        """Create a git checkpoint."""
        self.log(f"📌 Committing: {name}")
        subprocess.run(
            ["git", "add", "."],
            cwd=self.repo_root,
            check=False
        )
        subprocess.run(
            ["git", "commit", "-m", f"{name}"],
            cwd=self.repo_root,
            check=False
        )


def main():
    """Main entry point."""
    # Load config
    config_path = Path(__file__).parent / "config.json"
    with open(config_path) as f:
        config = json.load(f)
    
    repo_root = Path(__file__).parent.parent
    
    # Create orchestrator
    orchestrator = Orchestrator(repo_root, config)
    
    # Run a single improvement task
    improvement_made = orchestrator.run_single_improvement()
    
    if improvement_made:
        print("✅ Run completed - one code change merged")
    else:
        print("✅ Workflow complete - all tasks finished")


if __name__ == "__main__":
    main()
