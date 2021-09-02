Unpoweroutter
---
Makes your computer un-bad when it comes down to running out of battery. Instead of your computer just turning off like a weirdo it will instead give you a 60 second countdown (after many warnings) where you can go run and grab your power cable to plug your machine back in.

### Installing

Compile the binary
`$ cargo install --path .`

Do the systemd stuff so unpoweroutter starts with your computer
```
# MAKE SURE TO MODIFY unpoweroutter.service to include the correct path to your unpoweroutter binary
$ cp unpoweroutter.service /etc/systemd/system/unpoweroutter.service
$ systemctl start unpoweroutter
$ systemctl enable unpoweroutter
```
