import os

def read_repo(state: dict) -> dict:
    """
    Scans the repository path for Rust files and adds their content 
    to the state so the agent has context.
    """
    repo_path = state.get("repo_path")
    if not repo_path or not os.path.exists(repo_path):
        return {**state, "status": "error", "last_output": "Invalid repo path."}

    context_parts = []
    
    # Walk through the repo and collect .rs files
    for root, _, files in os.walk(repo_path):
        for file in files:
            if file.endswith(".rs"):
                full_path = os.path.join(root, file)
                rel_path = os.path.relpath(full_path, repo_path)
                try:
                    with open(full_path, "r", encoding="utf-8") as f:
                        content = f.read()
                        # Get file size in KB (metric)
                        size_kb = os.path.getsize(full_path) / 1024
                        context_parts.append(f"--- FILE: {rel_path} ({size_kb:.2f} KB) ---\n{content}\n")
                except Exception as e:
                    context_parts.append(f"--- FILE: {rel_path} (Error reading file: {e}) ---")

    full_context = "\n".join(context_parts)
    
    return {
        **state,
        "last_output": f"Read {len(context_parts)} Rust files.\n" + full_context,
        "status": "pending"
    }