# Syntx Grammar

program => declaration[] EOF

declaration => 
    (class_declaration) |
    fn_declaration |
    int_declaration |
    string_declaration |
    statement

fn_declaration => "fn" + identifier + "(" + params + ")" + block
int_declaration => "int" + identifier + "=" + expression + ";"
string_declaration => "str" + identifier + "=" + expression + ";"

statement =>
    expr_stmt |
    for_stmt |
    if_stmt |
    print_stmt |
    return_stmt |
    (while_stmt) |
    block

expr_stmt => expression + ";"
for_stmt => 
    int_declaration | string_declaration | None + ";" + expression? + ";" + expression? + block
if_stmt => "if" + expression + block + ("else" + block)?
print_stmt => "print" + expression + ";"
return_stmt => "return" + expression? + ";"
block => "{" + declaration[] + "}"

expression => identifier 

examples
```bofink
    print "hello world";

    str s1 = "hello";
    str s2 = "world";
    print s1 + " " s2 + "!";
```
