# Грамматика языка MiniC

## Содержание

1. [Обозначения](#обозначения)
2. [Программа](#программа)
3. [Препроцессор](#препроцессор)
4. [Объявления](#объявления)
5. [Типы](#типы)
6. [Инструкции](#инструкции)
7. [Выражения](#выражения)
8. [Приоритет операторов](#приоритет-операторов)
9. [Примеры](#примеры-программ)


## Обозначения

```
Символ    | Значение
----------|---------
{ ... }   | повторение 0 или более раз
[ ... ]   | опционально
( ... )   | группировка
|         | альтернатива
"..."     | терминальный символ
?...?     | специальная последовательность
```

## Программа

Программа состоит из последовательности объявлений и директив препроцессора.

```ebnf
Program = { Declaration | PreprocessorDirective } EOF;
```

## Препроцессор

Директивы препроцессора обрабатываются до основного синтаксического анализа.

```ebnf
PreprocessorDirective = "#define" Identifier [ Value ]
                      | "#undef" Identifier
                      | "#ifdef" Identifier Block
                      | "#ifndef" Identifier Block
                      | "#endif"
                      | "#else" Block;

Value = ? любой токен, кроме символа новой строки ?;
```

## Объявления

### Объявления верхнего уровня

```ebnf
Declaration = FunctionDecl | StructDecl | VarDecl;
```

### Функции

Функции объявляются с ключевым словом `fn` и могут возвращать значение.
Синтаксис включает поддержку стрелки `->` для указания возвращаемого типа.

```ebnf
FunctionDecl = "fn" Identifier "(" [ ParamList ] ")" [ "->" Type ] Block;

ParamList = Param { "," Param };
Param = Type Identifier;
```

**Примеры:**
```
fn main() { return 0; }                        // void функция
fn add(int a, int b) -> int { return a + b; }  // функция с возвращаемым типом
fn log(string msg) { print(msg); }             // void функция с параметром
```

### Структуры

Структуры объявляются с ключевым словом `struct` и могут содержать поля различных типов.

```ebnf
StructDecl = "struct" Identifier "{" { FieldDecl } "}";
FieldDecl = Type Identifier ";";
```

**Примеры:**
```
struct Point {
    int x;
    int y;
};

struct Person {
    string name;
    int age;
    bool active;
};

struct Circle {
    struct Point center;  // вложенная структура
    int radius;
};
```

### Переменные

Объявления переменных могут включать инициализатор.

```ebnf
VarDecl = Type Identifier [ "=" Expression ] ";";
```

**Примеры:**
```
int x = 42;
float pi = 3.14159;
string msg = "hello";
bool flag;
struct Point p;  // объявление структурной переменной
```

## Типы

### Базовые типы

```ebnf
Type = BasicType | StructType;

BasicType = "int" | "float" | "bool" | "void" | "string";
StructType = "struct" Identifier;
```

## Инструкции

### Блок

Блок группирует последовательность инструкций и создает новую область видимости.

```ebnf
Block = "{" { Statement } "}";
```

### Инструкции

```ebnf
Statement = VarDecl
          | ExprStmt
          | IfStmt
          | WhileStmt
          | ForStmt
          | ReturnStmt
          | Block
          | EmptyStmt;

ExprStmt = Expression ";";
EmptyStmt = ";";
```

### Условная инструкция

Поддерживается классический синтаксис `if-else` с обязательными скобками вокруг условия.
Проблема "висячего else" решается стандартным образом - `else` привязывается к ближайшему `if`.

```ebnf
IfStmt = "if" "(" Expression ")" Statement [ "else" Statement ];
```

**Примеры:**
```
if (x > 0) {
    return x;
}

if (x > 0) {
    return x;
} else {
    return -x;
}

if (a > b) {
    max = a;
} else if (a < b) {
    max = b;
} else {
    max = 0;
}
```

### Цикл while

Классический цикл с предусловием.

```ebnf
WhileStmt = "while" "(" Expression ")" Statement;
```

**Пример:**
```
while (i < 10) {
    sum = sum + i;
    i = i + 1;
}
```

### Цикл for

Гибкий цикл `for` с поддержкой различных комбинаций инициализации, условия и обновления.

```ebnf
ForStmt = "for" "(" ( VarDecl | ExprStmt | ";" ) 
                [ Expression ] ";" 
                [ Expression ] ")" Statement;
```

**Примеры:**
```
// Полная форма
for (int i = 0; i < 10; i = i + 1) {
    print(i);
}

// Без инициализации
for (; i < 10; i = i + 1) {
    print(i);
}

// Без условия (бесконечный цикл)
for (int i = 0;; i = i + 1) {
    if (i > 10) break;
}

// Пустой заголовок
for (;;) {
    break;
}
```

### Инструкция return

Возврат значения из функции. Для `void` функций может использоваться без значения.

```ebnf
ReturnStmt = "return" [ Expression ] ";";
```

**Примеры:**
```
return 42;
return;
return x + y;
```

## Выражения

Выражения строятся с учетом приоритета операторов (от низшего к высшему).
Все операторы бинарные, кроме специально отмеченных унарных.

### Уровень 9: Присваивание (правоассоциативное)

```ebnf
Expression = Assignment;
Assignment = LogicalOr { ("=" | "+=" | "-=" | "*=" | "/=") Assignment };
```

**Примеры:**
```
x = 5
x += y
a = b = c = 0
```

### Уровень 8: Логическое ИЛИ (левоассоциативное)

```ebnf
LogicalOr = LogicalAnd { "||" LogicalAnd };
```

**Пример:** `a || b || c`

### Уровень 7: Логическое И (левоассоциативное)

```ebnf
LogicalAnd = Equality { "&&" Equality };
```

**Пример:** `a && b && c`

### Уровень 6: Равенство (неассоциативное)

Операторы сравнения на равенство не могут быть сгруппированы без скобок.

```ebnf
Equality = Comparison { ("==" | "!=") Comparison };
```

**Пример:** `a == b != c` (требуется осторожность)

### Уровень 5: Сравнение (неассоциативное)

Операторы сравнения также неассоциативны.

```ebnf
Comparison = Additive { ("<" | "<=" | ">" | ">=") Additive };
```

**Пример:** `x < y <= z`

### Уровень 4: Сложение/вычитание (левоассоциативное)

```ebnf
Additive = Multiplicative { ("+" | "-") Multiplicative };
```

**Пример:** `a + b - c`

### Уровень 3: Умножение/деление/остаток (левоассоциативное)

```ebnf
Multiplicative = Unary { ("*" | "/" | "%") Unary };
```

**Пример:** `a * b / c % d`

### Уровень 2: Унарные операторы (правоассоциативные)

```ebnf
Unary = [ "!" | "-" | "+" ] Primary;
```

**Примеры:** `-x`, `!flag`, `+value`

### Уровень 1: Первичные выражения (высший приоритет)

```ebnf
Primary = Literal
        | Identifier
        | "(" Expression ")"
        | FunctionCall
        | StructAccess;

FunctionCall = Identifier "(" [ ArgList ] ")";
ArgList = Expression { "," Expression };

StructAccess = Primary "." Identifier;
```

**Примеры:**
```
42
"hello"
x
(1 + 2) * 3
foo(1, 2, 3)
point.x
obj.field.method()
```

## Литералы и идентификаторы

### Литералы

```ebnf
Literal = Integer | Float | String | Boolean;

Integer = [ "-" ] Digit { Digit };
Float = [ "-" ] Digit { Digit } "." Digit { Digit };
String = '"' { Character | EscapeSequence } '"';
Boolean = "true" | "false";
```

### Идентификаторы

```ebnf
Identifier = Letter { Letter | Digit | "_" };

Digit = "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9";
Letter = "A" | "B" | "C" | "D" | "E" | "F" | "G" | "H" | "I" | "J" 
       | "K" | "L" | "M" | "N" | "O" | "P" | "Q" | "R" | "S" | "T" 
       | "U" | "V" | "W" | "X" | "Y" | "Z"
       | "a" | "b" | "c" | "d" | "e" | "f" | "g" | "h" | "i" | "j" 
       | "k" | "l" | "m" | "n" | "o" | "p" | "q" | "r" | "s" | "t" 
       | "u" | "v" | "w" | "x" | "y" | "z";

EscapeSequence = "\\" ( "n" | "t" | "r" | "\\" | '"' | "'" );
Character = ? любой символ кроме '"', '\\', '\n', '\r' ?;
```

## Приоритет операторов

| Уровень | Категория       | Операторы                     | Ассоциативность |
|---------|-----------------|-------------------------------|-----------------|
| 1       | Первичные       | `()` `[]` `.`                 | левая           |
| 2       | Унарные         | `!` `-` `+`                   | правая          |
| 3       | Мультипликативные | `*` `/` `%`                 | левая           |
| 4       | Аддитивные      | `+` `-`                       | левая           |
| 5       | Сравнения       | `<` `<=` `>` `>=`             | неассоциативная |
| 6       | Равенство       | `==` `!=`                     | неассоциативная |
| 7       | Логическое И    | `&&`                          | левая           |
| 8       | Логическое ИЛИ  | `\|\|`                        | левая           |
| 9       | Присваивание    | `=` `+=` `-=` `*=` `/=`       | правая          |

## Примеры программ

### Пример 1: Простая программа
```c
fn main() {
    int x = 42;
    return x;
}
```

### Пример 2: Функция с параметрами
```c
fn max(int a, int b) -> int {
    if (a > b) {
        return a;
    } else {
        return b;
    }
}

fn main() {
    int m = max(10, 20);
    return m;
}
```

### Пример 3: Работа со структурами
```c
struct Point {
    int x;
    int y;
};

fn main() {
    struct Point p;
    p.x = 10;
    p.y = 20;
    return p.x + p.y;
}
```

### Пример 4: Циклы и препроцессор
```c
#define MAX 100

fn main() {
    int sum = 0;
    
    for (int i = 0; i < MAX; i = i + 1) {
        sum = sum + i;
    }
    
    return sum;
}
```

### Пример 5: Сложные выражения
```c
fn compute(int x, int y, int z, bool flag) -> int {
    int a = (x + y) * z - (x - y) / z;
    bool b = !flag && (x > y || z == 0);
    return a;
}
```

### Пример 6: Рекурсия
```c
fn factorial(int n) -> int {
    if (n <= 1) {
        return 1;
    }
    return n * factorial(n - 1);
}

fn main() {
    int result = factorial(5);
    return result;
}
```