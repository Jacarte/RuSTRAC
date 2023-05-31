import sys
import random



if __name__ == "__main__":
    f = sys.argv[1]
    f = open(f, "w")
    choices = "abcdefghijklmnopqrstuvwxyz"
    for i in range(1000000):
        f.write(random.choice(choices) + "\n")

