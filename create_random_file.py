import sys
import random



if __name__ == "__main__":
    f = sys.argv[1]
    f = open(f, "w")
    choices = "abcdefghijklmnopqrstuvwxyz"
    # 1 million symbols
    for i in range(2000000):
        symbol = "".join([random.choice(choices) for i in range(30)])
        f.write(random.choice(choices) + "\n")

