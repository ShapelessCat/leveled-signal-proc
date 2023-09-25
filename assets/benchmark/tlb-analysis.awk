#!/bin/awk -f

$1 == "processing_time_secs{op=\"process_event\"}" {
	s += $2
	c ++;
}
END {
	print s/c
}
