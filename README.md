# Bofink
Bofink is a simple typed scripting language. It is type checked at compile time.
## Getting started
coming soon...
## Examples
Hello world
```ts
print "Hello world!"; 
```

Typed variables
```ts
int i = 123;
str s = "a string";
```

Control flow
```ts
int i = 5 + 3;
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

// third param for custom increment
for i in 0:10:2 {
    print "bar";
}
```

Function declaration and usage
```ts
fun foo(param1: int, param2: str) {
    print "first param: " + p1;
    print "second param : " + p2;
}
test(5, "a string");
```
