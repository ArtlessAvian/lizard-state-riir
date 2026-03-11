
run-godot:
	just rust-workspace/build-gdext
	just godot-project/run

run-tui:
	just rust-workspace/run

run-godot-headless:
	just rust-workspace/build-gdext
	just godot-project/run-headless

export-windows:
	mkdir -pv export/windows/debug
	echo "TODO"

export-linux:
	mkdir -pv export/linux/debug
	echo "TODO"

export-web:
	mkdir -pv export/web/debug
	echo "TODO"

clean:
	just rust-workspace/clean
	rm -r export
