"""
# 📘 **Explanation for an LLM: How This Script Works**

## 🧠 **Purpose**
This script runs `cargo clippy` in JSON mode, filters the output to include **only Clippy warnings**, prints those warnings as raw JSON, and then exits with a code that indicates whether any warnings were found.

It is designed to be **cross‑platform** (Windows, Linux, macOS) and to support **arbitrary Clippy flags**.

---

# 📝 **Input Behavior**

### **1. Script invocation**
You run the script like this:

```
python filter_clippy_json.py
```

or with additional Clippy arguments:

```
python filter_clippy_json.py -- <extra clippy flags>
```

Everything after the literal `--` is passed directly to Clippy.

### **2. What the script runs internally**
The script always executes:

```
cargo clippy --all-targets --all-features --message-format=json -- <extra args>
```

The `--message-format=json` flag ensures that Clippy emits one JSON object per line.

### **3. What counts as a warning**
A JSON message is considered a Clippy warning if:

- It contains a `.message.code.code` field  
- That field starts with `"clippy::"`

All other JSON messages are ignored.

---

# 📤 **Output Behavior**

### **1. Standard Output (stdout)**
The script prints **only the JSON objects corresponding to Clippy warnings**, one per line, unchanged.

Example:

```
{"message": {... "code": {"code": "clippy::needless_borrow"} ...}}
{"message": {... "code": {"code": "clippy::map_unwrap_or"} ...}}
```

If there are **no warnings**, stdout is empty.

### **2. Standard Error (stderr)**
The script prints a short human‑readable message:

- If warnings exist:
  ```
  Found N Clippy warnings.
  ```

- If no warnings exist:
  ```
  No Clippy warnings.
  ```

### **3. Exit Code**
This is the key behavior:

| Condition | Exit Code | Meaning |
|----------|-----------|---------|
| **No Clippy warnings** | `0` | Success |
| **One or more Clippy warnings** | `1` | Failure |

This makes the script suitable for CI pipelines or automated gating.

---

# 📦 **How an LLM Should Interpret the Output**

### **If stdout is empty and exit code is 0**
- Clippy produced **no warnings**
- The codebase passes the lint check
- No further action is required

### **If stdout contains JSON objects and exit code is 1**
- Each line of stdout is a **Clippy warning**, in raw JSON format
- The LLM may parse these JSON objects to:
  - extract lint codes
  - identify file paths and spans
  - generate fixes
  - summarize issues
- The non‑zero exit code indicates that the warnings must be addressed

### **If stderr contains a message**
- It is informational only
- It should not be parsed as part of the lint data

---

# 🧩 **Example Interaction**

### Command:
```
python filter_clippy_json.py -- -W clippy::correctness
```

### stdout:
```
{"message": {... "code": {"code": "clippy::needless_borrow"} ...}}
{"message": {... "code": {"code": "clippy::unwrap_used"} ...}}
```

### stderr:
```
Found 2 Clippy warnings.
```

### exit code:
```
1
```

### Interpretation:
- Two warnings were found
- The JSON objects describe them
- The script signals failure via exit code 1

---

# 🎯 **Summary for an LLM**

- **Input:** Optional Clippy flags after `--`
- **Output:** Only JSON objects representing Clippy warnings
- **Exit code:**  
  - `0` → no warnings  
  - `1` → one or more warnings  
- **Use case:**  
  - Parse stdout to understand warnings  
  - Use exit code to decide pass/fail  
  - Ignore stderr except for human context  

---
"""
#!/usr/bin/env python3
import json
import subprocess
import sys

def main():
    # Extract extra Clippy args after "--"
    if "--" in sys.argv:
        idx = sys.argv.index("--")
        extra_args = sys.argv[idx + 1:]
    else:
        extra_args = []

    cmd = [
        "cargo", "clippy",
        "--all-targets",
        "--all-features",
        "--message-format=json",
        "--"
    ] + extra_args

    print("Running:", " ".join(cmd), file=sys.stderr)

    proc = subprocess.Popen(
        cmd,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1
    )

    warnings = []

    for line in proc.stdout:
        line = line.strip()
        if not line:
            continue

        try:
            msg = json.loads(line)
        except json.JSONDecodeError:
            continue

        # Keep only Clippy warnings
        code = msg.get("message", {}).get("code", {}).get("code")
        if code and code.startswith("clippy::"):
            warnings.append(msg)
            print(json.dumps(msg))  # preserve original JSON

    proc.wait()

    # Exit 1 if ANY warnings exist
    if warnings:
        print(f"\nFound {len(warnings)} Clippy warnings.", file=sys.stderr)
        sys.exit(1)

    # Otherwise success
    print("\nNo Clippy warnings.", file=sys.stderr)
    sys.exit(0)

if __name__ == "__main__":
    main()
