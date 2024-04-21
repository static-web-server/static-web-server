set terminal png size 1200,750 enhanced font "Sans,9"
set output 'overview.png'

set style data histogram
set style histogram clustered gap 1
set style fill solid
set boxwidth 0.9
set grid ytics

set title "Benchmark results"
set ylabel "Latency (ms)"

plot datafile using 2:xtic(1) title "min", \
     '' using 0:2:2 with labels offset -5.7,0.5 rotate left title "", \
     '' using 3 title "mean", \
     '' using 0:3:3 with labels offset -3.7,0.5 rotate left title "", \
     '' using 4 title "50th", \
     '' using 0:4:4 with labels offset -1.7,0.5 rotate left title "", \
     '' using 5 title "90th", \
     '' using 0:5:5 with labels offset 0.3,0.5 rotate left title "", \
     '' using 6 title "95th", \
     '' using 0:6:6 with labels offset 2.3,0.5 rotate left title "", \
     '' using 7 title "99th", \
     '' using 0:7:7 with labels offset 4.3,0.5 rotate left title "", \
     '' using 8 title "max", \
     '' using 0:8:8 with labels offset 6.3,0.5 rotate left title ""
