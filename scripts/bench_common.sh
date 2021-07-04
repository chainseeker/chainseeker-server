
# Usage: bench URL
bench() {
	autocannon $AUTOCANNON_FLAGS --workers $(cat /proc/cpuinfo | grep processors | wc -l) --no-progress $1 2>&1  | sed -r "s/\x1B\[([0-9]{1,3}(;[0-9]{1,2})?)?[mGK]//g"
}

