# Спецификация языка MiniC

## Оглавление

1. [Обзор языка](#обзор-языка)
2. [Лексическая структура](#лексическая-структура)
3. [Типы данных](#типы-данных)
4. [Выражения](#выражения)
5. [Операторы](#операторы)
6. [Управляющие конструкции](#управляющие-конструкции)
7. [Функции](#функции)
8. [Структуры](#структуры)
9. [Комментарии](#комментарии)
10. [Препроцессор](#препроцессор)
11. [Синтаксис](#синтаксис)
12. [Примеры программ](#примеры-программ)
13. [Справочная информация](#справочная-информация)

## Обзор языка

MiniC - это упрощенный C-подобный язык программирования, предназначенный для учебных целей. Язык сохраняет основные концепции языков семейства C, но значительно упрощен для облегчения реализации компилятора.

### Основные характеристики:
- Статическая типизация
- Процедурная парадигма
- Блочная структура
- Управление памятью: стековое размещение переменных
- Подмножество синтаксиса C
- Встроенный препроцессор с макросами и условной компиляцией

## Лексическая структура

### Кодировка
- Исходные файлы используют кодировку UTF-8
- Поддерживаются окончания строк: Unix (`\n`) и Windows (`\r\n`)

### Лексемы
```
Program = { Token } EOF

Token = Keyword | Identifier | Literal | Operator | Delimiter
```

**Важное замечание:** Комментарии удаляются препроцессором до лексического анализа.

### Ключевые слова (14)
```
Keyword = "if" | "else" | "while" | "for" | "int" | "float" | "bool"
| "void" | "return" | "true" | "false" | "struct" | "fn" | "string"
```

### Идентификаторы
```
Identifier  = Letter { Letter | Digit | "_" }
Letter      = "A" | "B" | ... | "Z" | "a" | ... | "z"
Digit       = "0" | "1" | ... | "9"
```

**Ограничения:**
- Максимальная длина: 255 символов
- Чувствительность к регистру: да
- Не могут совпадать с ключевыми словами
- Допустимые символы: ASCII буквы, цифры, подчеркивание
- Первый символ: буква или подчеркивание

**Примеры:**
```
x
_x
variable1
MAX_VALUE
is_valid
counter_123
```

### Литералы

#### Целые числа
```
Integer = [ "-" ] Digit { Digit }
```

**Диапазон:** `[-2³¹, 2³¹-1]` (32-битные целые со знаком)

**Примеры:**
```
0
42
-100
2147483647    // i32::MAX
-2147483648   // i32::MIN
```

#### Числа с плавающей точкой
```
Float = [ "-" ] Digit { Digit } "." Digit { Digit }
```

**Формат:** 64-битное число двойной точности (IEEE 754)

**Примеры:**
```
0.0
3.14
-2.71828
123.456
```

#### Строковые литералы
```
String = '"' { Character | EscapeSequence } '"'

EscapeSequence = "\\" ( "n" | "t" | "r" | "\\" | '"' | "'" )
Character = любой символ кроме '"', '\\', '\n', '\r'
```

**Особенности:**
- Escape-последовательности поддерживаются
- Многострочные строки не поддерживаются
- Максимальная длина: 65535 символов

**Примеры:**
```
""                    // Пустая строка
"hello"               // Простая строка
"Hello, world!"       // Строка с пробелами
"Line 1\nLine 2"      // С escape-последовательностями
"Quotes: \"text\""    // Кавычки внутри строки
```

#### Логические литералы
```
Boolean = "true" | "false"
```

### Операторы

#### Арифметические операторы
```
+   // Сложение
-   // Вычитание (бинарный) и отрицание (унарный)
*   // Умножение
/   // Деление
%   // Остаток от деления (только для целых чисел)
```

#### Операторы сравнения
```
==  // Равенство
!=  // Неравенство
<   // Меньше
<=  // Меньше или равно
>   // Больше
>=  // Больше или равно
```

#### Логические операторы
```
&&  // Логическое И
||  // Логическое ИЛИ
!   // Логическое НЕ (унарный)
```

#### Операторы присваивания
```
=   // Присваивание
+=  // Присваивание со сложением
-=  // Присваивание с вычитанием
*=  // Присваивание с умножением
/=  // Присваивание с делением
```

#### Специальные операторы
```
->  // Указание возвращаемого типа функции
.   // Доступ к полям структуры
```

### Разделители
```
(   )   // Круглые скобки
{   }   // Фигурные скобки
[   ]   // Квадратные скобки
;       // Точка с запятой
,       // Запятая
:       // Двоеточие
```

### Пробельные символы
```
Whitespace = " " | "\t" | "\n" | "\r"
```

Пробельные символы игнорируются, за исключением:
- Разделения токенов
- Определения границ строк
- Учета позиции для сообщений об ошибках

## Типы данных

### Базовые типы

#### Целочисленный тип (`int`)
- 32-битное целое со знаком
- Диапазон: `[-2³¹, 2³¹-1]`
- По умолчанию: не инициализируется (нужно явно инициализировать)

```c
int x = 42;
int y = -100;
int z;  // Неинициализированная переменная (значение не определено)
```

#### Тип с плавающей точкой (`float`)
- 64-битное число двойной точности (IEEE 754)
- По умолчанию: не инициализируется

```c
float pi = 3.14159;
float temperature = -273.15;
float ratio;  // Неинициализированная переменная
```

#### Логический тип (`bool`)
- Два значения: `true` и `false`
- По умолчанию: не инициализируется

```c
bool flag = true;
bool enabled = false;
bool status;  // Неинициализированная переменная
```

#### Пустой тип (`void`)
- Используется для функций, не возвращающих значение
- Не может быть типом переменной

#### Строковый тип (`string`)
- Последовательность символов UTF-8
- Используются двойные кавычки
- Поддерживает escape-последовательности

```c
string name = "MiniC";
string path = "C:\\projects\\minic";
```

### Производные типы

#### Структуры (`struct`)
Пользовательские составные типы:

```c
struct Point {
    int x;
    int y;
};

struct Person {
    string name;
    int age;
    bool active;
};
```

#### Указатели и массивы (запланировано в будущих спринтах)
```
int* ptr;        // Указатель на int
int arr[10];     // Массив из 10 int
```

## Выражения

### Арифметические выражения
```c
int a = 10 + 5;      // 15
int b = 20 - 3;      // 17
int c = 6 * 7;       // 42
int d = 100 / 4;     // 25
int e = 17 % 5;      // 2

float f = 3.14 * 2.0;  // 6.28
float g = 10.0 / 3.0;  // 3.333...
```

### Выражения сравнения
```c
bool b1 = (5 == 5);   // true
bool b2 = (10 != 5);  // true
bool b3 = (3 < 5);    // true
bool b4 = (7 > 2);    // true
bool b5 = (4 <= 4);   // true
bool b6 = (6 >= 9);   // false
```

### Логические выражения
```c
bool a = true && false;  // false
bool b = true || false;  // true
bool c = !true;          // false

// Комбинированные выражения
bool d = (x > 0) && (x < 10);
bool e = (status == "error") || (count == 0);
```

### Вызовы функций
```c
int result = add(5, 3);
print("result =", result);
int x = foo();
```

### Доступ к полям структур
```c
struct Point p;
p.x = 10;
p.y = p.x + 5;
```

## Операторы

### Оператор присваивания
```c
variable = expression;
```

**Составные операторы присваивания:**
```c
x += y;     // x = x + y
x -= y;     // x = x - y
x *= y;     // x = x * y
x /= y;     // x = x / y
```

### Приоритет операторов

| Приоритет | Операторы | Ассоциативность | Пример |
|-----------|-----------|-----------------|---------|
| 1 (высокий) | `()` `[]` `.` | слева направо | `(a + b) * c` |
| 2 | `!` `-` (унарный) `+` (унарный) | справа налево | `-x`, `!flag` |
| 3 | `*` `/` `%` | слева направо | `a * b / c` |
| 4 | `+` `-` | слева направо | `a + b - c` |
| 5 | `<` `<=` `>` `>=` | слева направо | `x < y <= z` |
| 6 | `==` `!=` | слева направо | `a == b != c` |
| 7 | `&&` | слева направо | `a && b && c` |
| 8 | `||` | слева направо | `a || b || c` |
| 9 (низкий) | `=` `+=` `-=` `*=` `/=` | справа налево | `a = b = c` |

**Пример:**
```c
int result = 2 + 3 * 4;       // 14, а не 20 (умножение имеет более высокий приоритет)
bool check = x > 0 && x < 10; // && имеет меньший приоритет чем >, поэтому x > 0 вычисляется первым
```

## Управляющие конструкции

### Условный оператор `if-else`
```c
if (condition) {
    // выполняется если condition == true
}

if (condition) {
    // выполняется если condition == true
} else {
    // выполняется если condition == false
}

if (condition1) {
    // выполняется если condition1 == true
} else if (condition2) {
    // выполняется если condition2 == true
} else {
    // выполняется если все условия == false
}
```

### Цикл `while`
```c
while (condition) {
    // выполняется пока condition == true
}
```

### Цикл `for`
```c
for (initialization; condition; increment) {
    // тело цикла
}

// Пример:
for (int i = 0; i < 10; i = i + 1) {
    // выполнится 10 раз
}
```

### Оператор `break`
Прерывает выполнение цикла:
```c
while (true) {
    if (condition) {
        break;  // выход из цикла
    }
}
```

## Функции

### Объявление функции
```c
return_type function_name(parameter_list) {
    // тело функции
    [return expression;]
}
```

### Типы функций

#### Функции, возвращающие значение
```c
int add(int a, int b) {
    return a + b;
}

float calculate_area(float radius) {
    return 3.14159 * radius * radius;
}

bool is_even(int number) {
    return (number % 2) == 0;
}
```

#### Функции, не возвращающие значение (`void`)
```c
void print_hello() {
    // ... вывод на экран
    return;  // опционально
}

void initialize() {
    // инициализация
    // return не требуется
}
```

### Синтаксис со стрелкой
Для функций можно использовать альтернативный синтаксис с указанием возвращаемого типа через `->`:

```c
fn add(int a, int b) -> int {
    return a + b;
}

fn main() {
    return 0;
}
```

### Параметры функции
```c
// Без параметров
void no_params() { ... }

// Один параметр
void single_param(int x) { ... }

// Несколько параметров
int multiple_params(int a, float b, bool c) { ... }

// Параметры передаются по значению (копируются)
```

### Вызов функции
```c
// Вызов без возвращаемого значения
print_hello();

// Вызов с возвращаемым значением
int sum = add(5, 3);
float area = calculate_area(2.5);
bool result = is_even(7);
```

### Функция `main`
Точка входа программы:

```c
fn main() {
    // код программы
    return 0;
}
```

**Особенности:**
- Использует ключевое слово `fn` вместо типа возвращаемого значения
- Всегда возвращает `int` (неявно 0 если нет return)
- Не принимает параметров (в базовой версии)
- Вызывается автоматически при запуске программы

## Структуры

### Определение структуры
```c
struct Name {
    type1 field1;
    type2 field2;
    // ...
};
```

**Примеры:**
```c
struct Point {
    int x;
    int y;
};

struct Rectangle {
    struct Point top_left;    // Использование структуры как типа
    struct Point bottom_right;
    float area;
    bool visible;
};
```

### Объявление переменных структуры
```c
struct Point p1;                      // Объявление
struct Point p2 = {10, 20};           // Инициализация
```

### Доступ к полям структуры
```c
struct Point p;
p.x = 10;
p.y = 20;

int current_x = p.x;
int current_y = p.y;
```

### Структуры как параметры функций
```c
// Функция вычисления расстояния между двумя точками
float distance(struct Point a, struct Point b) {
    int dx = b.x - a.x;
    int dy = b.y - a.y;
    return sqrt(dx*dx + dy*dy);  // sqrt запланирована в будущих спринтах
}
```

## Комментарии

### Однострочные комментарии
```c
// Это однострочный комментарий
int x = 42;  // Комментарий после кода
// Можно комментировать целые строки
```

### Многострочные комментарии
```c
/* Это многострочный комментарий
   который может занимать
   несколько строк */

/* Комментарий может быть /* вложенным */ в MiniC */
```

**Особенности:**
- Комментарии удаляются препроцессором до лексического анализа
- Не могут быть вложены в строковые литералы
- Многострочные комментарии поддерживают вложенность

## Препроцессор

MiniC включает простой препроцессор, обрабатывающий директивы перед компиляцией.

### Директивы препроцессора

#### `#define` - определение макроса
```c
#define MAX_SIZE 100
#define GREETING "Hello, World!"
#define DEBUG 1

int array[MAX_SIZE];  // Заменяется на int array[100];
string msg = GREETING; // Заменяется на string msg = "Hello, World!";
```

#### `#undef` - удаление макроса
```c
#define TEMP_VALUE 42
#undef TEMP_VALUE
// TEMP_VALUE больше не определен
```

#### `#ifdef` / `#ifndef` / `#endif` - условная компиляция
```c
#ifdef DEBUG
    log("Debug mode enabled");  // Включится только если DEBUG определен
#endif

#ifndef RELEASE
    int debug_counter = 0;      // Включится только если RELEASE не определен
#endif
```

#### `#else` - альтернативная ветвь условной компиляции
```c
#ifdef FEATURE_A
    enable_feature_a();
#else
    enable_default_feature();
#endif
```

### Особенности препроцессора MiniC:
1. Макросы поддерживают только простую замену (без аргументов)
2. Обнаруживает рекурсивные определения макросов
3. Обрабатывает вложенные условные директивы
4. Сохраняет нумерацию строк для корректных сообщений об ошибках

## Синтаксис

### Полная грамматика в EBNF

```ebnf
Program         = { Declaration | PreprocessorDirective } EOF;

PreprocessorDirective = "#define" Identifier [ Value ]
                      | "#undef" Identifier
                      | "#ifdef" Identifier Block
                      | "#ifndef" Identifier Block
                      | "#endif"
                      | "#else" Block;

Declaration     = FunctionDecl | StructDecl | VarDecl;

FunctionDecl    = "fn" Identifier "(" [ ParamList ] ")" [ "->" Type ] Block;
ParamList       = Param { "," Param };
Param           = Type Identifier;

StructDecl      = "struct" Identifier "{" { FieldDecl } "}";
FieldDecl       = Type Identifier ";";

VarDecl         = Type Identifier [ "=" Expression ] ";";

Type            = BasicType | StructType;
BasicType       = "int" | "float" | "bool" | "void" | "string";
StructType      = "struct" Identifier;

Block           = "{" { Statement } "}";

Statement       = VarDecl
                | ExprStmt
                | IfStmt
                | WhileStmt
                | ForStmt
                | ReturnStmt
                | Block
                | EmptyStmt;

ExprStmt        = Expression ";";
EmptyStmt       = ";";

IfStmt          = "if" "(" Expression ")" Statement [ "else" Statement ];
WhileStmt       = "while" "(" Expression ")" Statement;
ForStmt         = "for" "(" ( VarDecl | ExprStmt | ";" )
                        [ Expression ] ";"
                        [ Expression ] ")" Statement;
ReturnStmt      = "return" [ Expression ] ";";

Expression      = Assignment;
Assignment      = LogicalOr { ("=" | "+=" | "-=" | "*=" | "/=") Assignment };
LogicalOr       = LogicalAnd { "||" LogicalAnd };
LogicalAnd      = Equality { "&&" Equality };
Equality        = Comparison { ("==" | "!=") Comparison };
Comparison      = Additive { ("<" | "<=" | ">" | ">=") Additive };
Additive        = Multiplicative { ("+" | "-") Multiplicative };
Multiplicative  = Unary { ("*" | "/" | "%") Unary };
Unary           = [ "!" | "-" | "+" ] Primary;
Primary         = Literal
                | Identifier
                | "(" Expression ")"
                | FunctionCall
                | StructAccess;

FunctionCall    = Identifier "(" [ ArgList ] ")";
ArgList         = Expression { "," Expression };

StructAccess    = Primary "." Identifier;

Literal         = Integer | Float | String | Boolean;
Integer         = [ "-" ] Digit { Digit };
Float           = [ "-" ] Digit { Digit } "." Digit { Digit };
String          = '"' { Character | EscapeSequence } '"';
Boolean         = "true" | "false";

Identifier      = Letter { Letter | Digit | "_" };
Digit           = "0" | "1" | ... | "9";
Letter          = "A" | ... | "Z" | "a" | ... | "z";
EscapeSequence  = "\\" ( "n" | "t" | "r" | "\\" | '"' | "'" );
```

## Примеры программ

### Пример 1: Простая программа с препроцессором
```c
// Определение констант через препроцессор
#define MAX_VALUE 100
#define WELCOME_MSG "Hello from MiniC!"

fn main() {
    int counter = 0;
    
    // Использование макросов
    while (counter < MAX_VALUE) {
        counter = counter + 1;
    }
    
    string message = WELCOME_MSG;
    return 0;
}
```

### Пример 2: Функции и условия
```c
// Вычисление факториала
fn factorial(int n) -> int {
    if (n <= 1) {
        return 1;
    }
    return n * factorial(n - 1);
}

fn main() {
    int result = factorial(5);  // 120
    return 0;
}
```

### Пример 3: Циклы и структуры
```c
// Работа с геометрическими фигурами
struct Point {
    int x;
    int y;
};

struct Circle {
    struct Point center;
    int radius;
};

fn main() {
    // Создание и инициализация структур
    struct Point origin = {0, 0};
    struct Circle circle = {origin, 5};
    
    // Доступ к полям
    circle.center.x = 10;
    circle.center.y = 20;
    
    return circle.radius;
}
```

### Пример 4: Полная программа
```c
/*
 * Калькулятор площадей фигур
 * Демонстрирует все основные возможности языка
 */

#define PI 3.14159

struct Circle {
    struct Point {
        int x;
        int y;
    } center;
    int radius;
};

fn circle_area(struct Circle c) -> float {
    return PI * c.radius * c.radius;
}

fn main() {
    struct Circle c;
    c.center.x = 0;
    c.center.y = 0;
    c.radius = 5;
    
    float area = circle_area(c);
    return 0;
}
```

## Справочная информация

### Формат сообщений об ошибках
```
file.src:line:column: type: message
```

**Примеры:**
```
program.src:10:25: error: Неожиданный символ '@'
program.src:15:5: error: Незавершенная строковая константа
program.src:20:12: warning: Неиспользуемая переменная 'temp'
program.src:25:8: error: отсутствует ';' после return
```

### Типы ошибок
1. **Лексические ошибки** - недопустимые символы, незавершенные строки
2. **Ошибки препроцессора** - проблемы с макросами и директивами
3. **Синтаксические ошибки** - нарушение правил грамматики
4. **Семантические ошибки** - нарушение правил типов (в будущем)

### Зарезервированные слова (14)
```
if      else    while   for
int     float   bool    void    string
return  true    false
struct  fn
```

### Операторы (по приоритету)
1. `()` `[]` `.`
2. `!` `-` (унарный) `+` (унарный)
3. `*` `/` `%`
4. `+` `-`
5. `<` `<=` `>` `>=`
6. `==` `!=`
7. `&&`
8. `||`
9. `=` `+=` `-=` `*=` `/=`

### Escape-последовательности
```
\n    Новая строка (line feed)
\t    Горизонтальная табуляция
\r    Возврат каретки
\\    Обратный слеш
\"    Двойная кавычка
\'    Одиночная кавычка
```

### Директивы препроцессора
```
#define   // Определение макроса
#undef    // Удаление макроса
#ifdef    // Если макрос определен
#ifndef   // Если макрос не определен
#endif    // Конец условного блока
#else     // Альтернативная ветвь условия
```

**Версия спецификации:** 2.0
**Дата последнего обновления:** 4.03.2026