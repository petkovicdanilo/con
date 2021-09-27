#!/bin/sh

user=$( who | awk '{print $1}' )

for cgroup in cpu memory pids
do
    path=/sys/fs/cgroup/$cgroup/con
    mkdir $path
    chown -R $user:$user $path
    echo 1 > $path/notify_on_release
done
