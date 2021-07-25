# Qor Basic

Qor Basic is a simple BASIC interpreter for the Qor kernel.

## Commands

### `goto`

Goto the given line number.

Syntax: `goto {line}`

### `if`

Conditionally run a statement.

Syntax: `if {cond} then {statement}`

### `list`

List the currently stored program.

Syntax: `list`

### `load`

Load a program from disk.

Syntax: `load {filename}`

### `print`

Print constants or variables.

Syntax: `print [{value} ... ]`

### `run`

Run the currently stored program.

Syntax: `run`

### `store`

Store the currently stored program to disk.

Syntax: `store {filename}`

## Variables

Variables can be any string beginning with a letter or an underscore, followed by from 0 to 255 alpha-numeric characters or underscores.

A variable can be assigned to using the following syntax:

```
var = {expr}
```

## Values

Qor BASIC uses two datatypes, numbers (integers) and strings.

## Line Input

To write a line into the program memory, use a number preceeding the statement, for example:

```
10 print "Hello World"
```
## Example Program

The following program prints out the prime numbers up to 100.

```
10       print "Primes"
20       A = 2
30       print A
40       A = A + 1
50       if A > 100 then goto 500
60       M = 2
70       if A - (A / M) * M == 0 then goto 40
80       M = M + 1
90       if M >= A then goto 30
100      goto 70
```
