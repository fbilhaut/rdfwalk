CAST = demo.cast
TMP_GIF = demo.tmp.gif
GIF = demo.gif

demo: $(GIF)

$(CAST):
	asciinema rec -c "cargo run -- http://localhost:7878/query" $(CAST)

$(TMP_GIF): $(CAST)
	agg --speed 3 $(CAST) $(TMP_GIF)

$(GIF): $(TMP_GIF)
	gifsicle -O3 --lossy=80 --resize-width 1024 $(TMP_GIF) -o $(GIF)

.PHONY: demo
