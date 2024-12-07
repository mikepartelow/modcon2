#!/usr/bin/env python3

import os
import shlex
import subprocess
import sys

CAKE_HOME=os.path.join(os.path.abspath(os.getcwd()), "cake.d")

def main() -> int:
    if len(sys.argv) < 2:
        print("Usage: cake <command> [arguments]")
        return 1

    command = sys.argv[1]
    parts = shlex.split(command)

    if len(parts) < 1:
        print("Invalid command")
        return 1

    sub_name = parts[0]
    sub_path = os.path.join(CAKE_HOME, sub_name)

    sub_args = parts[1:]

    if not os.path.isfile(sub_path):
        print(f"No such subcommand: {sub_name} ({sub_path})")
        return 1

    try:
        cp = subprocess.run([sub_path] + sub_args, check=True, cwd=os.getcwd())
        return cp.returncode
    except subprocess.CalledProcessError as e:
        print(f"Error executing script: {e}")
        return 1

if __name__ == "__main__":
    sys.exit(main())
