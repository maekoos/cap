INPUT_FILE:=./resources/cube.cap

all: compile apply

compile:
	cargo run $(INPUT_FILE)

apply:
	cd apply-mccmd && cargo run ../out.mccmd

reset:
	cd apply-mccmd && cargo run ../del.mccmd
