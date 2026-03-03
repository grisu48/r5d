# r5d (remind)

Customizable reminders in configuration (and other) files.

`r5d` will search for `!remind` statements in configuration files and fire a reminder when a defined time has passed.
This utility allows to add notification and alarm clocks directly into configuration files.

`!remind` statements look like the following:

```
!remind
!remind - Reminders flagged with '-' will always fire
# !remind 3000-01-01T00:00:00 Reminders can be in comments

!noremind Stop processing reminders.
!remind - This one will not be seen anymore.
```

Reminders can start anywhere in a line. This allows the program to work against various configuration files, where reminders might be put as comments.

When at least one reminder is found, the program will report it and exit with return code 5.