# Bofink
Bofink is a simple typed scripting language. It is type checked at compile time.
## Getting started
coming soon...
## Examples
Hello world
```ts
print "Hello world!";
```

Type inference
```ts
let i = 123; // i64
let s = "a string"; // string
```
Immutability-by-default
```ts
let i = 1;
i = 2; // Error! Cannot mutate non-mutable variable
```

Mutable variables
```ts
mut i = 1;
i = 2;
```

Control flow
```ts
let i = 5 + 3;
if i > 6 {
    print "i is greater than 6";
}
```

For-loops
```ts
// iterates from 0 to 2
for i in 0:3 {
    print "foo";
}
// foo
// foo
// foo

// third param for custom increment
for i in 0:10:2 {
    print "bar" + i;
}
// bar0
// bar2
// bar4
// bar6
// bar8
```

Function declaration and usage
```ts
fun foo(param1: int, param2: str) {
    print "first param: " + param1;
    print "second param : " + param2;
}
foo(5, "a string");
```
