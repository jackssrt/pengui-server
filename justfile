[default]
run:
	cargo run

db:
	echo IF THIS IS PRODUCTION PRESS CONTROL C NOWWWWW
	docker compose down db --remove-orphans
	# evil
	sudo rm -rf db/data 
	docker compose up db

fmt:
	cargo fmt