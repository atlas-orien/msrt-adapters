# msrt-embassy-uart 架构设计（Phase 1）

## 目标

`msrt-embassy-uart` 是 MSRT 协议在 Embassy UART 环境下的适配层。

它不实现协议本身，也不重新设计可靠传输逻辑。真正的协议状态机仍然由 `msrt::Engine` 负责。

这个 crate 的目标是把 `msrt::Engine` 的底层接口组合成 MCU 更容易使用的 UART 驱动接口。

## 核心结论

可靠传输不能把 `send` 和 `receive` 当成两个互相独立的操作。

即使应用层只想发送 message，协议层也必须持续接收对端返回的 ACK，否则无法知道：

- 哪些 packet 已经成功到达
- 哪些 packet 需要重发
- 什么时候可以释放发送状态
- 什么时候应该上报发送失败

因此 `msrt-embassy-uart` 的核心不是 `send().await` 或 `receive().await`，而是一个持续推进协议状态的 `poll_once`。

## 设计原则

Phase 1 使用显式 `poll_once` 模型。

原因：

- 适合 MCU 主循环
- 不隐藏任务调度
- 不强依赖 Embassy executor 的 channel / mutex / static storage 策略
- 更容易移植到 RTIC、裸 loop、自研 scheduler 或其他 no-std runtime
- 保持 MSRT Engine 的真实执行过程可见，方便调试可靠传输

未来可以在 `poll_once` 之上再包装 `run().await` / handle API，但 `poll_once` 应该是最底层、最稳定的能力。

## 边界

`msrt-embassy-uart` 负责：

- 持有 UART 对象
- 持有 `msrt::Engine`
- 从 UART 读取 bytes
- 把 bytes 交给 `engine.receive`
- 把当前时间交给 `engine.tick`
- 调用 `engine.poll_event`
- 遇到 `Event::Write` 时写回 UART
- 遇到 `Event::Message` 时缓存完整 message
- 遇到 `Event::SendFailed` 时缓存或返回发送失败状态
- 把 UART 错误和 MSRT 错误统一映射成 adapter error

`msrt-embassy-uart` 不负责：

- 定义 MSRT wire format
- 定义可靠传输算法
- 分配 message id
- 处理 packet ack/retransmit 细节
- 绑定某一个具体 MCU HAL
- 内置 Embassy task spawning 策略
- 内置 DMA / interrupt / ring buffer 策略

## 核心对象

`UartAdapter<Uart>` 拥有：

- `uart: Uart`
- `engine: msrt::Engine`
- 接收完成的 message 缓存
- 发送失败状态缓存

`UartAdapter` 是一个单所有权对象。

Phase 1 不引入共享 handle，不引入内部锁，也不引入后台 task。用户可以把它放进自己的主循环、Embassy task、RTIC task 或其他调度模型中。

## Public API 草案

Phase 1 推荐的核心 API：

```rust
impl<Uart> UartAdapter<Uart> {
    pub const fn new(uart: Uart, engine: msrt::Engine) -> Self;

    pub fn send_message(&mut self, message: &[u8]) -> Result<msrt::core::MessageId>;

    pub async fn poll_once(&mut self, now_ms: u64, rx_buf: &mut [u8]) -> Result<()>;

    }
```

其中：

- `send_message` 只表示提交一条待发送 message
- `poll_once` 才是真正推进协议状态的函数
- `poll_once_dispatch` 用 handler 分发完整 message 和发送失败事件

## send_message 语义

`send_message` 不是“同步发送完成”。

它只是把应用层 message 提交给 `msrt::Engine`：

```text
application message
    -> send_message
    -> engine.send
    -> engine 内部生成待发送 packet 状态
```

真正把 packet 写入 UART、等待 ACK、触发重发、确认完成，都发生在后续的 `poll_once` 中。

因此：

- `send_message` 不等待 ACK
- `send_message` 不保证对端已经收到 message
- `send_message` 不直接读 UART
- `send_message` 不直接完成可靠传输

这个设计可以避免在 no-std MCU 里隐藏阻塞行为。

## poll_once 语义

`poll_once` 是 adapter 的核心。

每次调用时，它执行一次有限的协议推进：

```text
1. 尝试从 UART 读取一段 bytes
2. 如果读到 bytes，则调用 engine.receive(bytes)
3. 调用 engine.tick(now_ms)
4. 循环调用 engine.poll_event()
5. 如果事件是 Event::Write，则写入 UART
6. 如果事件是 Event::Message，则缓存 message
7. 如果事件是 Event::SendFailed，则缓存失败状态
```

`poll_once` 不应该成为无限循环。

无限循环应该由用户或未来的高级包装层决定：

```rust
loop {
    adapter.poll_once(now_ms(), &mut rx_buf).await?;

    adapter.poll_once_dispatch(
        now_ms(),
        &mut rx_buf,
        |message| app.handle_message(message),
        |error| app.handle_error(error),
    ).await;
}
```

## 为什么不直接使用 run().await

`run().await` 对用户体验更好，但它会引入更多设计问题：

- 是否内部无限循环
- 如何从其他 task 提交 message
- 是否需要 channel
- channel buffer 多大
- message buffer 谁拥有
- 是否需要 mutex
- 是否绑定 Embassy executor
- 如何处理 backpressure

这些问题不是 MSRT 协议本身的问题，而是 runtime 集成问题。

所以 Phase 1 先冻结 `poll_once` 作为最小稳定边界。

未来可以在这个基础上增加：

```rust
adapter.run(callbacks).await
handle.send_message(message).await
handle.recv_message().await
```

但它们应该是上层便利封装，而不是最底层协议驱动能力。

## 使用模型

典型 MCU 使用方式：

```rust
let mut adapter = UartAdapter::new(uart, engine);
let mut rx_buf = [0u8; 128];

adapter.send_message(b"hello")?;

loop {
    adapter.poll_once(now_ms(), &mut rx_buf).await?;

    adapter.poll_once_dispatch(
        now_ms(),
        &mut rx_buf,
        |message| app.handle_message(message),
        |error| app.handle_error(error),
    ).await;
}
```

这个模型里，应用层只需要理解：

- `send_message`：提交消息
- `poll_once`：推进协议
- `poll_once_dispatch`：推进协议并分发 message/error

## 和 msrt::Engine 的关系

`msrt::Engine` 是协议状态机。

`msrt-embassy-uart::UartAdapter` 是 I/O adapter。

关系如下：

```text
Application
    |
    | send_message / poll_once_dispatch
    v
UartAdapter
    |
    | engine.send / engine.receive / engine.tick / engine.poll_event
    v
msrt::Engine
    |
    | Event::Write
    v
UART
```

`UartAdapter` 不应该把协议细节泄漏给应用层，但也不应该隐藏 MCU 调度模型。

## 当前非目标

Phase 1 暂不处理：

- Embassy task handle API
- 多 producer 发送队列
- rx/tx split ownership
- DMA buffer 策略
- interrupt-driven ring buffer
- 多 message 接收队列
- 板级示例
- 硬件性能测试

这些属于后续阶段。

## 下一步

代码需要从当前的 `send().await` / `receive().await` 模型调整为：

- `send_message`
- `poll_once`
- `poll_once_dispatch`

同时删除让用户误解为“send 已经可靠发送完成”的 API 命名。
