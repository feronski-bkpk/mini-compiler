section .text
global print_int
global print_string
global read_int
global exit

; ============================================================
; exit - завершение программы
exit:
    mov rax, 60          ; syscall номер для exit
    syscall
    ; never returns

; ============================================================
; print_int - печать целого числа
print_int:
    push rbp
    mov rbp, rsp
    sub rsp, 32

    mov rax, rdi
    mov rsi, rsp
    add rsi, 20
    mov byte [rsi], 0

    test rax, rax
    jns .positive

    neg rax
    mov byte [rsp], '-'
    inc rsp
    jmp .convert

.positive:
    cmp rax, 0
    jne .convert
    dec rsi
    mov byte [rsi], '0'
    jmp .print

.convert:
    dec rsi
    xor rdx, rdx
    mov rcx, 10
    div rcx
    add dl, '0'
    mov [rsi], dl
    test rax, rax
    jnz .convert

.print:
    mov rdx, rsp
    add rdx, 20
    sub rdx, rsi
    mov rcx, rdx

    mov rax, 1
    mov rdi, 1
    mov rdx, rcx
    syscall

    mov rsp, rbp
    pop rbp
    ret

; ============================================================
; print_string - печать строки
print_string:
    push rbp
    mov rbp, rsp

    mov rsi, rdi
    xor rcx, rcx
    dec rcx
.count:
    inc rcx
    cmp byte [rsi + rcx], 0
    jne .count

    mov rax, 1
    mov rdi, 1
    mov rdx, rcx
    syscall

    pop rbp
    ret

; ============================================================
; read_int - чтение целого числа
read_int:
    push rbp
    mov rbp, rsp
    sub rsp, 32

    mov rax, 0
    mov rdi, 0
    mov rsi, rsp
    mov rdx, 31
    syscall

    mov rsi, rsp
    xor rax, rax
    xor rcx, rcx

.skip_spaces:
    cmp byte [rsi], ' '
    je .next_char
    cmp byte [rsi], 9
    je .next_char
    jmp .check_sign

.next_char:
    inc rsi
    jmp .skip_spaces

.check_sign:
    cmp byte [rsi], '-'
    jne .parse_digit
    mov rcx, 1
    inc rsi
    jmp .parse_digit

.parse_digit:
    movzx rdx, byte [rsi]
    cmp dl, '0'
    jb .done
    cmp dl, '9'
    ja .done

    sub dl, '0'
    imul rax, rax, 10
    add rax, rdx

    inc rsi
    jmp .parse_digit

.done:
    test rcx, rcx
    jz .positive
    neg rax

.positive:
    mov rsp, rbp
    pop rbp
    ret

section .note.GNU-stack noalloc noexec nowrite progbits