// src/main.rs

mod alloc;

#[global_allocator]
static GLOBAL: alloc::HardenedAlloc = alloc::HardenedAlloc;

fn main() {
    // 從這一刻起，你的程式中所有需要「堆分配」的標準操作，
    // 例如創建一個 Vec、String、Box，或者使用 println! (內部也可能需要分配記憶體)，
    // 都會自動通過我們註冊的 GLOBAL 配置器，最終呼叫到 hardened_malloc 的實現。

    println!("Kronos starting up, using hardened_malloc.");

    // 讓我們來觸發一次堆分配
    let numbers: Vec<i32> = (0..10).collect();
    println!("A vector allocated on the hardened heap: {:?}", numbers);

    // 你可以繼續編寫你的 kronos 核心邏輯...
}
