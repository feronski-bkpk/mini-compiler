//! Позиция в исходном коде.
//!
//! Представляет собой пару (строка, колонка) для указания местоположения
//! в исходном коде. Используется для:
//! - Отслеживания позиции токенов
//! - Формирования сообщений об ошибках
//! - Подсветки синтаксиса в IDE
//!
//! # Индексация
//!
//! Используется 1-индексирование для удобства восприятия человеком:
//! - Первая строка: 1
//! - Первая колонка: 1
//!
//! # Пример
//!
//! ```
//! use minic::common::position::Position;
//!
//! let pos = Position::new(10, 25);
//! println!("Ошибка на позиции {}", pos);
//! ```

/// Позиция в исходном коде.
///
/// Хранит номер строки и колонки, начиная с 1.
///
/// # Примеры
///
/// ```
/// use minic::common::position::Position;
///
/// let pos1 = Position::new(5, 10);
/// assert_eq!(pos1.line, 5);
/// assert_eq!(pos1.column, 10);
///
/// let start = Position::start();
/// assert_eq!(start.line, 1);
/// assert_eq!(start.column, 1);
///
/// let mut pos = Position::start();
/// pos.advance_column(3);
/// pos.new_line();
/// assert_eq!(pos.line, 2);
/// assert_eq!(pos.column, 1);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    /// Номер строки (начинается с 1)
    pub line: usize,

    /// Номер колонки (начинается с 1)
    pub column: usize,
}

impl Position {
    /// Создает новую позицию с указанными строкой и колонкой.
    ///
    /// # Аргументы
    ///
    /// * `line` - номер строки (≥ 1)
    /// * `column` - номер колонки (≥ 1)
    ///
    /// # Пример
    ///
    /// ```
    /// use minic::common::position::Position;
    ///
    /// let pos = Position::new(10, 25);
    /// ```
    pub fn new(line: usize, column: usize) -> Self {
        debug_assert!(line >= 1, "Номер строки должен быть ≥ 1");
        debug_assert!(column >= 1, "Номер колонки должен быть ≥ 1");

        Self { line, column }
    }

    /// Возвращает стартовую позицию (1:1).
    ///
    /// Соответствует началу файла.
    ///
    /// # Пример
    ///
    /// ```
    /// use minic::common::position::Position;
    ///
    /// let start = Position::start();
    /// assert_eq!(start.line, 1);
    /// assert_eq!(start.column, 1);
    /// ```
    pub fn start() -> Self {
        Self::new(1, 1)
    }

    /// Увеличивает номер колонки на указанное значение.
    ///
    /// # Аргументы
    ///
    /// * `by` - количество символов для продвижения
    ///
    /// # Пример
    ///
    /// ```
    /// use minic::common::position::Position;
    ///
    /// let mut pos = Position::new(1, 5);
    /// pos.advance_column(3);
    /// assert_eq!(pos.column, 8);
    /// ```
    pub fn advance_column(&mut self, by: usize) {
        self.column += by;
    }

    /// Переходит на новую строку.
    ///
    /// Увеличивает номер строки на 1 и сбрасывает колонку в 1.
    ///
    /// # Пример
    ///
    /// ```
    /// use minic::common::position::Position;
    ///
    /// let mut pos = Position::new(1, 10);
    /// pos.new_line();
    /// assert_eq!(pos.line, 2);
    /// assert_eq!(pos.column, 1);
    /// ```
    pub fn new_line(&mut self) {
        self.line += 1;
        self.column = 1;
    }

    /// Создает копию позиции с увеличенной колонкой.
    ///
    /// Полезно для вычисления позиции конца токена.
    ///
    /// # Аргументы
    ///
    /// * `offset` - смещение колонки
    ///
    /// # Пример
    ///
    /// ```
    /// use minic::common::position::Position;
    ///
    /// let start = Position::new(1, 5);
    /// let end = start.with_column_offset(10);
    /// assert_eq!(end.column, 15);
    /// ```
    pub fn with_column_offset(&self, offset: usize) -> Self {
        Self {
            line: self.line,
            column: self.column + offset,
        }
    }

    /// Проверяет, является ли позиция валидной.
    ///
    /// Позиция считается валидной, если оба индекса ≥ 1.
    ///
    /// # Возвращает
    ///
    /// `true` если позиция валидна, `false` в противном случае
    pub fn is_valid(&self) -> bool {
        self.line >= 1 && self.column >= 1
    }

    /// Возвращает позицию в формате для отладки.
    ///
    /// Формат: `(строка:колонка)`
    pub fn debug(&self) -> String {
        format!("({}:{})", self.line, self.column)
    }
}

impl std::fmt::Display for Position {
    /// Форматирует позицию для вывода.
    ///
    /// Формат: `строка:колонка`
    ///
    /// # Пример
    ///
    /// ```
    /// use minic::common::position::Position;
    ///
    /// let pos = Position::new(10, 25);
    /// assert_eq!(pos.to_string(), "10:25");
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

impl Default for Position {
    /// Возвращает позицию по умолчанию (1:1).
    fn default() -> Self {
        Self::start()
    }
}

/// Реализация оператора сложения для Position.
///
/// Позволяет сдвигать позицию по колонкам.
impl std::ops::Add<usize> for Position {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self {
            line: self.line,
            column: self.column + rhs,
        }
    }
}

/// Реализация оператора вычитания для Position.
///
/// Позволяет сдвигать позицию по колонкам назад.
/// Убедитесь, что результат не становится меньше 1.
impl std::ops::Sub<usize> for Position {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self::Output {
        let new_column = if self.column > rhs {
            self.column - rhs
        } else {
            1
        };

        Self {
            line: self.line,
            column: new_column,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_creation() {
        let pos = Position::new(10, 20);
        assert_eq!(pos.line, 10);
        assert_eq!(pos.column, 20);
    }

    #[test]
    fn test_position_start() {
        let pos = Position::start();
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 1);
    }

    #[test]
    fn test_advance_column() {
        let mut pos = Position::new(5, 10);
        pos.advance_column(3);
        assert_eq!(pos.column, 13);
        assert_eq!(pos.line, 5);
    }

    #[test]
    fn test_new_line() {
        let mut pos = Position::new(3, 15);
        pos.new_line();
        assert_eq!(pos.line, 4);
        assert_eq!(pos.column, 1);
    }

    #[test]
    fn test_with_column_offset() {
        let pos = Position::new(2, 5);
        let new_pos = pos.with_column_offset(10);
        assert_eq!(new_pos.line, 2);
        assert_eq!(new_pos.column, 15);
    }

    #[test]
    fn test_display() {
        let pos = Position::new(7, 12);
        assert_eq!(pos.to_string(), "7:12");
    }

    #[test]
    fn test_addition() {
        let pos = Position::new(1, 5);
        let new_pos = pos + 10;
        assert_eq!(new_pos.column, 15);
    }

    #[test]
    fn test_subtraction() {
        let pos = Position::new(1, 15);
        let new_pos = pos - 10;
        assert_eq!(new_pos.column, 5);

        let pos2 = Position::new(1, 3);
        let new_pos2 = pos2 - 5;
        assert_eq!(new_pos2.column, 1);
    }

    #[test]
    fn test_is_valid() {
        let valid_pos = Position::new(1, 1);
        assert!(valid_pos.is_valid());

        let invalid_pos = Position { line: 0, column: 1 };
        assert!(!invalid_pos.is_valid());
    }
}
