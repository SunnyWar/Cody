"""
Console output utilities for Windows compatibility.
"""

def safe_print(msg: str):
    """Print with Windows console unicode fallback.
    
    Windows PowerShell console (cp1252 encoding) cannot display many
    unicode characters including emoji. This function attempts to print
    the full message, falling back to ASCII-only if encoding fails.
    """
    try:
        print(msg)
    except UnicodeEncodeError:
        # Fall back to ASCII-only for Windows console
        print(msg.encode('ascii', 'ignore').decode('ascii'))
