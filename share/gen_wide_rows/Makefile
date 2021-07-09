all: main

main: main.c
	gcc $< -o $@ -lcfitsio -Wall -Wextra

run: main
	./main

clean:
	rm main
