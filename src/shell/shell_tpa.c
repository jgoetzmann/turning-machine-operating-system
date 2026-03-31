int gi;

int dispatch_line() {
    int j;
    int k;
    int end;
    gi = 0;
    while (lineget(gi) == 32) {
        gi = gi + 1;
    }
    if (lineget(gi) == 0) {
        return 0;
    }
    if (lineget(gi) == 99 && lineget(gi + 1) == 99) {
        if (lineget(gi + 2) != 32) {
            putchar(63);
            putchar(10);
        } else {
            j = gi + 3;
            while (lineget(j) == 32) {
                j = j + 1;
            }
            end = j;
            while (lineget(end) != 0) {
                end = end + 1;
            }
            end = end - 1;
            while (end >= j && lineget(end) == 32) {
                end = end - 1;
            }
            if (end < j) {
                ccend();
            } else {
                k = j;
                while (k <= end) {
                    namech(lineget(k));
                    k = k + 1;
                }
                ccend();
            }
        }
        return 0;
    }
    if (lineget(gi) == 99 && lineget(gi + 1) == 108 && lineget(gi + 2) == 115) {
        j = gi + 3;
        while (lineget(j) == 32) {
            j = j + 1;
        }
        if (lineget(j) != 0) {
            putchar(63);
            putchar(10);
        } else {
            putchar(27);
            putchar(91);
            putchar(50);
            putchar(74);
            putchar(27);
            putchar(91);
            putchar(72);
        }
        return 0;
    }
    if (lineget(gi) == 100 && lineget(gi + 1) == 105 && lineget(gi + 2) == 114) {
        j = gi + 3;
        while (lineget(j) == 32) {
            j = j + 1;
        }
        if (lineget(j) != 0) {
            putchar(63);
            putchar(10);
        } else {
            listdir();
        }
        return 0;
    }
    if (lineget(gi) == 100 && lineget(gi + 1) == 101 && lineget(gi + 2) == 108) {
        if (lineget(gi + 3) != 32) {
            putchar(63);
            putchar(10);
        } else {
            j = gi + 4;
            while (lineget(j) == 32) {
                j = j + 1;
            }
            end = j;
            while (lineget(end) != 0) {
                end = end + 1;
            }
            end = end - 1;
            while (end >= j && lineget(end) == 32) {
                end = end - 1;
            }
            if (end < j) {
                delend();
            } else {
                k = j;
                while (k <= end) {
                    namech(lineget(k));
                    k = k + 1;
                }
                delend();
            }
        }
        return 0;
    }
    if (lineget(gi) == 104 && lineget(gi + 1) == 97 && lineget(gi + 2) == 108 && lineget(gi + 3) == 116) {
        j = gi + 4;
        while (lineget(j) == 32) {
            j = j + 1;
        }
        if (lineget(j) != 0) {
            putchar(63);
            putchar(10);
            return 0;
        }
        puts("HALT");
        return 1;
    }
    if (lineget(gi) == 104 && lineget(gi + 1) == 101 && lineget(gi + 2) == 108 && lineget(gi + 3) == 112) {
        j = gi + 4;
        while (lineget(j) == 32) {
            j = j + 1;
        }
        if (lineget(j) != 0) {
            putchar(63);
            putchar(10);
        } else {
            puts("dir type halt help mem cls run del cc");
        }
        return 0;
    }
    if (lineget(gi) == 109 && lineget(gi + 1) == 101 && lineget(gi + 2) == 109) {
        j = gi + 3;
        while (lineget(j) == 32) {
            j = j + 1;
        }
        if (lineget(j) != 0) {
            putchar(63);
            putchar(10);
        } else {
            puts("0000-00FF BIOS\n0100-3FFF TPA\n4000-7FFF KERNEL\n8000-BFFF SHELL\nC000-EFFF FS\nF000-FDFF STACK\nFE00-FEFF IO\nFF00-FFFF META");
        }
        return 0;
    }
    if (lineget(gi) == 114 && lineget(gi + 1) == 117 && lineget(gi + 2) == 110) {
        if (lineget(gi + 3) != 32) {
            putchar(63);
            putchar(10);
        } else {
            j = gi + 4;
            while (lineget(j) == 32) {
                j = j + 1;
            }
            end = j;
            while (lineget(end) != 0) {
                end = end + 1;
            }
            end = end - 1;
            while (end >= j && lineget(end) == 32) {
                end = end - 1;
            }
            if (end < j) {
                runend();
            } else {
                k = j;
                while (k <= end) {
                    namech(lineget(k));
                    k = k + 1;
                }
                runend();
            }
        }
        return 0;
    }
    if (lineget(gi) == 116 && lineget(gi + 1) == 121 && lineget(gi + 2) == 112 && lineget(gi + 3) == 101) {
        if (lineget(gi + 4) != 32) {
            putchar(63);
            putchar(10);
        } else {
            j = gi + 5;
            while (lineget(j) == 32) {
                j = j + 1;
            }
            end = j;
            while (lineget(end) != 0) {
                end = end + 1;
            }
            end = end - 1;
            while (end >= j && lineget(end) == 32) {
                end = end - 1;
            }
            if (end < j) {
                namend();
            } else {
                k = j;
                while (k <= end) {
                    namech(lineget(k));
                    k = k + 1;
                }
                namend();
            }
        }
        return 0;
    }
    putchar(63);
    putchar(10);
    return 0;
}

int main() {
    while (1) {
        puts("A> ");
        readline();
        if (dispatch_line()) {
            return 0;
        }
    }
    return 0;
}
