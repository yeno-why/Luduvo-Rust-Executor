BITS 64

struc shared_memory
    .enabled: resb 1
    .pad: resb 7
    .old: resq 1
    .luau_compile: resq 1
    .luau_load: resq 1
    .defer: resq 1
    .lua_gettop: resq 1
    .lua_settop: resq 1
    .script: resq 1
    .script_len: resq 1
endstruc

executor_hook:
    push rbx
    push rsi
    push rdi
    push r12
    sub rsp, 0x38

    mov rsi, rcx ; rsi = L
    movabs rbx, 0x1151721933 ; rbx = shared_memory

    ; lua_gettop(L)
    mov rcx, rsi
    call [rbx + shared_memory.lua_gettop]
    mov r12d, eax

    ; if not enabled return old else set enabled ti fasle and run script
    test [rbx + shared_memory.enabled], 1
    jz .return
    mov [rbx + shared_memory.enabled], 0

    ; is ther a scirpt?
    mov rax, [rbx + shared_memory.script]
    test rax, rax
    jz .return
    
    ; luau_compile(src, len, NULL, &outsize)
    mov rcx, [rbx + shared_memory.script]
    mov rdx, [rbx + shared_memory.script_len]
    xor r8d, r8d
    lea r9,  [rsp + 0x20]
    mov [rsp + 0x20], 0
    call [rbx + shared_memory.luau_compile]

    ; if failed return old
    test rax, rax
    jz .return
    mov rdi, rax

    ; luau_load(L, chunk_name, bytecode, size, 0)
    mov rcx, rsi
    lea rdx, [rel chunk_name]
    mov r8,  rdi
    mov r9,  [rsp + 0x20]
    mov [rsp + 0x20], 0
    call [rbx + shared_memory.luau_load]

    ; if failed return old
    test eax, eax
    jnz .return

    ; defer(L, 0, top+1, "task.defer", 0.0)
    mov rcx, rsi
    xorps xmm1, xmm1
    xor edx, edx
    lea r8d, [r12 + 1]
    lea r9,  [rel defer_tag]
    call [rbx + shared_memory.defer]

.return:

    ; lua_settop(L, top)
    mov rcx, rsi
    mov edx, r12d
    call [rbx + shared_memory.lua_settop]

    mov rcx, rsi
    mov r11, [rbx + shared_memory.old]
    add r11, 0x67

    add rsp, 0x38
    pop r12
    pop rdi
    pop rsi
    pop rbx
    jmp r11

chunk_name: db "@rust", 0
defer_tag: db "task.defer", 0