import sys

# filter result file: inlcude only LaTeX rows
def main():
    with open(sys.argv[1]) as f:
        for line in f:
            line = line.strip()
            if line.startswith("\\href{"):
                print(line)

if __name__ == '__main__':
    if len(sys.argv) != 2:
        print("Usage: python filter.py <results file>")
        sys.exit(1)

    main()
