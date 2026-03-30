int main() {
    int line[129];
    int len;
    int c;
    int i;
    int j;
    int k;
    int end;
    while (1) {
        puts("A> ");
        len = 0;
        while (1) {
            c = getchar();
            if (c == 0) {
                return 0;
            }
            if (c == 8 || c == 127) {
                if (len > 0) {
                    len = len - 1;
                    putchar(8);
                    putchar(32);
                    putchar(8);
                }
            } else if (c == 10 || c == 13) {
                line[len] = 0;
                break;
            } else {
                if (len < 128) {
                    line[len] = c;
                    len = len + 1;
                }
            }
        }
        i = 0;
        while (line[i] == 32) {
            i = i + 1;
        }
        if (line[i] == 0) {
        } else if (line[i] == 99 && line[i + 1] == 99) {
            if (line[i + 2] != 32) {
                putchar(63);
                putchar(10);
            } else {
                j = i + 3;
                while (line[j] == 32) {
                    j = j + 1;
                }
                end = j;
                while (line[end] != 0) {
                    end = end + 1;
                }
                end = end - 1;
                while (end >= j && line[end] == 32) {
                    end = end - 1;
                }
                if (end < j) {
                    ccend();
                } else {
                    k = j;
                    while (k <= end) {
                        namech(line[k]);
                        k = k + 1;
                    }
                    ccend();
                }
            }
        } else if (line[i] == 99 && line[i + 1] == 108 && line[i + 2] == 115) {
            j = i + 3;
            while (line[j] == 32) {
                j = j + 1;
            }
            if (line[j] != 0) {
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
        } else if (line[i] == 100 && line[i + 1] == 105 && line[i + 2] == 114) {
            j = i + 3;
            while (line[j] == 32) {
                j = j + 1;
            }
            if (line[j] != 0) {
                putchar(63);
                putchar(10);
            } else {
                listdir();
            }
        } else if (line[i] == 100 && line[i + 1] == 101 && line[i + 2] == 108) {
            if (line[i + 3] != 32) {
                putchar(63);
                putchar(10);
            } else {
                j = i + 4;
                while (line[j] == 32) {
                    j = j + 1;
                }
                end = j;
                while (line[end] != 0) {
                    end = end + 1;
                }
                end = end - 1;
                while (end >= j && line[end] == 32) {
                    end = end - 1;
                }
                if (end < j) {
                    delend();
                } else {
                    k = j;
                    while (k <= end) {
                        namech(line[k]);
                        k = k + 1;
                    }
                    delend();
                }
            }
        } else if (line[i] == 104 && line[i + 1] == 97 && line[i + 2] == 108 && line[i + 3] == 116) {
            j = i + 4;
            while (line[j] == 32) {
                j = j + 1;
            }
            if (line[j] != 0) {
                putchar(63);
                putchar(10);
            } else {
                puts("HALT");
                return 0;
            }
        } else if (line[i] == 104 && line[i + 1] == 101 && line[i + 2] == 108 && line[i + 3] == 112) {
            j = i + 4;
            while (line[j] == 32) {
                j = j + 1;
            }
            if (line[j] != 0) {
                putchar(63);
                putchar(10);
            } else {
                puts("dir type halt help mem cls run del cc");
            }
        } else if (line[i] == 109 && line[i + 1] == 101 && line[i + 2] == 109) {
            j = i + 3;
            while (line[j] == 32) {
                j = j + 1;
            }
            if (line[j] != 0) {
                putchar(63);
                putchar(10);
            } else {
                puts("0000-00FF BIOS\n0100-3FFF TPA\n4000-7FFF KERNEL\n8000-BFFF SHELL\nC000-EFFF FS\nF000-FDFF STACK\nFE00-FEFF IO\nFF00-FFFF META");
            }
        } else if (line[i] == 114 && line[i + 1] == 117 && line[i + 2] == 110) {
            if (line[i + 3] != 32) {
                putchar(63);
                putchar(10);
            } else {
                j = i + 4;
                while (line[j] == 32) {
                    j = j + 1;
                }
                end = j;
                while (line[end] != 0) {
                    end = end + 1;
                }
                end = end - 1;
                while (end >= j && line[end] == 32) {
                    end = end - 1;
                }
                if (end < j) {
                    runend();
                } else {
                    k = j;
                    while (k <= end) {
                        namech(line[k]);
                        k = k + 1;
                    }
                    runend();
                }
            }
        } else if (line[i] == 116 && line[i + 1] == 121 && line[i + 2] == 112 && line[i + 3] == 101) {
            if (line[i + 4] != 32) {
                putchar(63);
                putchar(10);
            } else {
                j = i + 5;
                while (line[j] == 32) {
                    j = j + 1;
                }
                end = j;
                while (line[end] != 0) {
                    end = end + 1;
                }
                end = end - 1;
                while (end >= j && line[end] == 32) {
                    end = end - 1;
                }
                if (end < j) {
                    namend();
                } else {
                    k = j;
                    while (k <= end) {
                        namech(line[k]);
                        k = k + 1;
                    }
                    namend();
                }
            }
        } else {
            putchar(63);
            putchar(10);
        }
    }
    return 0;
}
