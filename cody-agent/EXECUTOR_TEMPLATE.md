# Standard Executor Template

This template shows the canonical pattern used by all executors.
Reference: `refactoring_executor.py`, `performance_executor.py`

## Function Signature
```python
def execute_task(item_id: str, repo_root: Path, config: dict) -> bool:
    """Execute a specific task using the LLM."""
```

## Standard Flow

### 1. Load TODO Item
```python
todo_list = TodoList("category", repo_root)

item = None
for todo_item in todo_list.items:
    if todo_item.id == item_id:
        item = todo_item
        break

if not item:
    print(f"❌ Item {item_id} not found in TODO list")
    return False

if item.status == "completed":
    print(f"⏭️ Item {item_id} is already completed")
    return True

todo_list.mark_in_progress(item_id)
todo_list.save()
```

### 2. Gather File Content
```python
file_path = item.metadata.get("file") or item.files_affected[0]
full_path = repo_root / file_path

if not full_path.exists():
    print(f"❌ File not found: {full_path}")
    return False

file_content = full_path.read_text(encoding="utf-8")
```

### 3. Build Prompt
```python
prompt = (
    f"Fix/implement the following task:\n\n"
    f"Task: {item.title}\n"
    f"Description: {item.description}\n"
    f"File: {file_path}\n\n"
    f"Instructions:\n"
    f"- Return ONLY a single ```rust code block.\n"
    f"- The first non-empty line must be a comment with the file path: // {file_path}\n"
    f"- Include the FULL updated file content.\n\n"
    f"Current file content:\n\n{file_content}\n"
)
```

### 4. Call LLM
```python
response = call_ai(prompt, config)
```

### 5. Extract File Content
```python
def extract_file_content(response: str) -> tuple[str, str]:
    """Extract file path and content from LLM response.
    Returns (file_path, content) or (None, None) if extraction fails.
    """
    if "```rust" in response:
        start = response.find("```rust") + 7
        end = response.find("```", start)
        if end != -1:
            code = response[start:end].strip()
            lines = code.split("\n")
            file_path = None
            
            # Extract file path from comment
            for line in lines[:5]:
                if "//" in line and (".rs" in line or "/" in line):
                    path_part = line.split("//", 1)[1].strip()
                    if path_part and not path_part.startswith(" "):
                        file_path = path_part
                    break
            
            # Remove comment lines from code
            code_lines = []
            skip_comments = True
            for line in lines:
                if skip_comments and line.strip().startswith("//"):
                    continue
                skip_comments = False
                code_lines.append(line)
            
            content = "\n".join(code_lines).strip()
            return file_path, content
    
    return None, None

response_file_path, new_content = extract_file_content(response)

if not response_file_path or not new_content:
    print("❌ LLM response did not include updated file content")
    return False
```

### 6. Apply Changes
```python
def apply_code_changes(repo_root: Path, file_path: str, new_content: str) -> bool:
    """Write new content directly to file."""
    try:
        full_path = repo_root / file_path
        if not full_path.parent.exists():
            print(f"❌ Parent directory does not exist: {full_path.parent}")
            return False
        
        full_path.write_text(new_content, encoding="utf-8")
        print(f"✅ Updated {file_path}")
        return True
        
    except Exception as e:
        print(f"❌ Error writing file: {e}")
        return False

if not apply_code_changes(repo_root, file_path, new_content):
    return False
```

### 7. Mark Complete
```python
todo_list.mark_completed(item_id)
todo_list.save()
print(f"\n✅ Task {item_id} completed successfully")
return True
```

## Helper: call_ai
```python
def call_ai(prompt: str, config: dict) -> str:
    """Call the AI with the prompt."""
    if config.get("use_local"):
        client = OpenAI(
            api_key="ollama",
            base_url=config.get("api_base", "http://localhost:11434/v1"),
            timeout=3600.0
        )
    else:
        api_key = os.environ.get("OPENAI_API_KEY")
        if not api_key:
            print("\n❌ Error: OPENAI_API_KEY environment variable not set")
            sys.exit(1)
        client = OpenAI(api_key=api_key, timeout=3600.0)

    model = config["model"]
    print(f"🤖 Implementing with {model}...")

    response = client.chat.completions.create(
        model=model,
        messages=[
            {
                "role": "system",
                "content": "You are a senior Rust engineer. Return only the full, updated file content in a single rust code block. The first non-empty line must be a comment with the file path."
            },
            {"role": "user", "content": prompt}
        ],
        temperature=0.2
    )

    return response.choices[0].message.content
```

## Key Points
1. **Always** mark TODO item in-progress before starting
2. **Always** read full file content (not snippets)
3. **Always** instruct LLM to return full file in code block
4. **Always** extract file path from comment in LLM response
5. **Always** mark completed on success
6. **Never** parse JSON from LLM for code (use code blocks)
