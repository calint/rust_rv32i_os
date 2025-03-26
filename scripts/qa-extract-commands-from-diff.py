#!/bin/python
import sys
import os
import re


def extract_text(filename):
    """Extracts text after '>' from lines in a file, trimmed."""
    try:
        with open(filename, "r") as file:
            for line in file:
                match = re.search(r">\s*(.*)", line)
                if match:
                    trimmed_text = match.group(1).strip()
                    print(trimmed_text)
    except FileNotFoundError:
        print(f"Error: File '{filename}' not found")
    except Exception as e:
        print(f"An error occurred: {e}")


if __name__ == "__main__":
    os.chdir(os.path.dirname(os.path.abspath(sys.argv[0])))
    extract_text("../qa/test.diff")
