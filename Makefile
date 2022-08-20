DC = docker compose

dc_build:
	${DC} build

dc_up:
	${DC} up -d

dc_down:
	${DC} down

dc_ps:
	${DC} ps

dc_logs:
	${DC} logs

dc_login:
	${DC} exec -it web bash