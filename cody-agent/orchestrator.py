"""
Master Orchestrator for Cody AI Improvement Workflow

Coordinates the complete workflow:
1. Refactoring analysis → execution → loop until done
2. Performance analysis → execution → loop until done  
3. Features analysis → execute up to 3 items (rerun 1&2 if large diff)

This script manages the multi-step automated improvement process.
"""

import os
import sys
import json
import subprocess
from pathlib import Path
from datetime import datetime
from typing import List, Tuple

# Import our modules
from todo_manager import TodoList
import refactoring_analyzer
import refactoring_executor
import performance_analyzer
import performance_executor
import features_analyzer
import features_executor


class Orchestrator:
    """Master orchestrator for the improvement workflow."""
    
    def __init__(self, repo_root: Path, config: dict):
        self.repo_root = repo_root
        self.config = config
        self.log_file = repo_root / "orchestrator.log"
        
    def log(self, message: str):
        """Log a message to both console and file."""
        timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        log_line = f"[{timestamp}] {message}"
        print(log_line)
        
        with open(self.log_file, 'a') as f:
            f.write(log_line + "\n")
    
    def create_checkpoint(self, name: str):
        """Create a git checkpoint."""
        self.log(f"📌 Creating checkpoint: {name}")
        subprocess.run(
            ["git", "add", "."],
            cwd=self.repo_root,
            check=False
        )
        subprocess.run(
            ["git", "commit", "-m", f"Checkpoint: {name}"],
            cwd=self.repo_root,
            check=False
        )
    
    def step_1_refactoring(self) -> bool:
        """Step 1: Refactoring analysis and execution loop."""
        self.log("\n" + "=" * 70)
        self.log("STEP 1: REFACTORING")
        self.log("=" * 70)
        
        # 1a. Analysis
        self.log("\n1a. Running refactoring analysis...")
        added = refactoring_analyzer.analyze(self.repo_root, self.config)
        self.log(f"   Added {added} refactoring opportunities")
        
        # 1b. Execution loop
        self.log("\n1b. Executing refactoring tasks...")
        todo_list = TodoList("refactoring", self.repo_root)
        
        completed = 0
        while True:
            next_item = todo_list.get_next_item()
            if not next_item:
                self.log("   ✅ All refactoring tasks completed")
                break
            
            self.log(f"\n   Working on: {next_item.id} - {next_item.title}")
            success = refactoring_executor.execute_refactoring(
                next_item.id, 
                self.repo_root, 
                self.config
            )
            
            if success:
                completed += 1
                self.create_checkpoint(f"Refactoring: {next_item.id}")
            else:
                self.log(f"   ❌ Failed to complete {next_item.id}")
                # Continue with other items
            
            # Reload TODO list
            todo_list = TodoList("refactoring", self.repo_root)
        
        self.log(f"\nStep 1 complete: {completed} refactorings applied")
        return completed > 0
    
    def step_2_performance(self) -> bool:
        """Step 2: Performance analysis and execution loop."""
        self.log("\n" + "=" * 70)
        self.log("STEP 2: PERFORMANCE OPTIMIZATION")
        self.log("=" * 70)
        
        # 2a. Analysis
        self.log("\n2a. Running performance analysis...")
        added = performance_analyzer.analyze(self.repo_root, self.config)
        self.log(f"   Added {added} performance opportunities")
        
        # 2b. Execution loop
        self.log("\n2b. Executing performance optimizations...")
        todo_list = TodoList("performance", self.repo_root)
        
        completed = 0
        while True:
            next_item = todo_list.get_next_item()
            if not next_item:
                self.log("   ✅ All performance tasks completed")
                break
            
            self.log(f"\n   Working on: {next_item.id} - {next_item.title}")
            success = performance_executor.execute_optimization(
                next_item.id,
                self.repo_root,
                self.config
            )
            
            if success:
                completed += 1
                self.create_checkpoint(f"Performance: {next_item.id}")
            else:
                self.log(f"   ❌ Failed to complete {next_item.id}")
            
            # Reload TODO list
            todo_list = TodoList("performance", self.repo_root)
        
        self.log(f"\nStep 2 complete: {completed} optimizations applied")
        return completed > 0
    
    def step_3_features(self, max_features: int = 3) -> int:
        """Step 3: Features analysis and limited execution."""
        self.log("\n" + "=" * 70)
        self.log("STEP 3: WORLD-CLASS FEATURES")
        self.log("=" * 70)
        
        # 3a. Analysis
        self.log("\n3a. Running features analysis...")
        added = features_analyzer.analyze(self.repo_root, self.config)
        self.log(f"   Added {added} feature opportunities")
        
        # 3b. Execute up to max_features items
        self.log(f"\n3b. Executing up to {max_features} features...")
        todo_list = TodoList("features", self.repo_root)
        
        completed = 0
        large_diff_count = 0
        
        for i in range(max_features):
            next_item = todo_list.get_next_item()
            if not next_item:
                self.log("   ℹ️ No more features available")
                break
            
            self.log(f"\n   Working on feature {i+1}/{max_features}: {next_item.id} - {next_item.title}")
            success, diff_size = features_executor.execute_feature(
                next_item.id,
                self.repo_root,
                self.config
            )
            
            if success:
                completed += 1
                self.create_checkpoint(f"Feature: {next_item.id}")
                
                # Check if diff is large
                if diff_size == "large":
                    large_diff_count += 1
                    self.log(f"   📏 Large diff detected for {next_item.id}")
                    
                    # Rerun steps 1 and 2
                    self.log("\n   🔄 Large diff detected, rerunning refactoring and performance...")
                    self.step_1_refactoring()
                    self.step_2_performance()
            else:
                self.log(f"   ❌ Failed to complete {next_item.id}")
            
            # Reload TODO list
            todo_list = TodoList("features", self.repo_root)
        
        self.log(f"\nStep 3 complete: {completed} features implemented ({large_diff_count} triggered refactor/perf cycles)")
        return completed
    
    def validate_todos(self):
        """Validate existing TODO items are still relevant."""
        self.log("\n" + "=" * 70)
        self.log("VALIDATING TODO LISTS")
        self.log("=" * 70)
        
        for category in ["refactoring", "performance", "features"]:
            self.log(f"\nValidating {category} TODO list...")
            todo_list = TodoList(category, self.repo_root)
            
            # Simple validation: just report stats
            total = len(todo_list.items)
            not_started = todo_list.count_by_status("not-started")
            in_progress = todo_list.count_by_status("in-progress")
            completed = todo_list.count_by_status("completed")
            
            self.log(f"  Total: {total}, Not Started: {not_started}, In Progress: {in_progress}, Completed: {completed}")
    
    def run_full_workflow(self):
        """Execute the complete improvement workflow."""
        self.log("\n" + "#" * 70)
        self.log("# CODY AI IMPROVEMENT ORCHESTRATOR")
        self.log(f"# Started: {datetime.now()}")
        self.log("#" * 70)
        
        try:
            # Ensure we're on main and clean
            self.log("\n🔧 Preparing workspace...")
            subprocess.run(["git", "checkout", "main"], cwd=self.repo_root, check=False)
            
            # Validate existing TODOs
            self.validate_todos()
            
            # Run the workflow
            self.step_1_refactoring()
            self.step_2_performance()
            features_completed = self.step_3_features(max_features=3)
            
            # Final summary
            self.log("\n" + "#" * 70)
            self.log("# WORKFLOW COMPLETE")
            self.log(f"# Finished: {datetime.now()}")
            self.log("#" * 70)
            
            # Print TODO stats
            self.log("\n📊 Final TODO Statistics:")
            for category in ["refactoring", "performance", "features"]:
                todo_list = TodoList(category, self.repo_root)
                self.log(f"\n{category.upper()}:")
                self.log(f"  Total: {len(todo_list.items)}")
                self.log(f"  Not Started: {todo_list.count_by_status('not-started')}")
                self.log(f"  In Progress: {todo_list.count_by_status('in-progress')}")
                self.log(f"  Completed: {todo_list.count_by_status('completed')}")
            
            self.log("\n✅ Orchestrator finished successfully")
            
        except KeyboardInterrupt:
            self.log("\n⚠️ Workflow interrupted by user")
            sys.exit(1)
        except Exception as e:
            self.log(f"\n❌ Workflow failed with error: {e}")
            import traceback
            self.log(traceback.format_exc())
            sys.exit(1)


def main():
    """Main entry point."""
    # Load config
    config_path = Path(__file__).parent / "config.json"
    with open(config_path) as f:
        config = json.load(f)
    
    repo_root = Path(__file__).parent.parent
    
    # Create orchestrator
    orchestrator = Orchestrator(repo_root, config)
    
    # Run the workflow
    orchestrator.run_full_workflow()


if __name__ == "__main__":
    main()
