# UI—Preview—GPU 呈现交接契约

2026-10-30当前实现核对，PreviewPort／FrameDescriptor版本2。旧版本与逐轮恢复设计已[原样归档](archive/2026-10-30-pre-review/README.md)，详细对象／方法与trait关系见[系统图](review/SYSTEM_MAP.md)。

## 三个边界

| 边界 | 当前接口 | 不传递的权力 |
| --- | --- | --- |
| application→宿主适配 | PreviewPort::request(&SkinProject, FrameSource)→FrameDescriptor | 不传可变项目、egui／wgpu句柄或保存职责 |
| UI→原生呈现 | UI定义Presenter，NativeHost实现；current／paint／thumbnail | 不传可变渲染器、项目写入口或历史身份 |
| 宿主→GPU／后台 | FrameWorker、Plan、GpuComposer及带租约Draw | 不把TextureId／FrameKey当资源所有权 |

FrameSource含HistorySessionId、ExpectedState(head／revision)和PreviewContext：目标Document／Components／Workspace、草稿完整状态、尺寸／视口代次、场景与模拟时间。FrameDescriptor给不透明FrameKey、来源及Pending／Ready；没有外部截图传输服务或真实stable场景保证。

## 准备、激活与显示

1. ProjectSession选择当前已提交或准确匹配的草稿项目，校验目标／视口，提交只读请求。
2. NativeHost保留期望来源；FrameWorker准备脱离／只读的图像数据与合成工作，资源先准入再分配。
3. 任务／GPU完成回执必须与完整期望来源及设备状态匹配；取消、历史、场景、resize／DPI或新草稿使旧候选不可激活。没有目标时清除期望和可呈现旧帧，不靠UI稍后刷新来失效。
4. 新帧准备成功后切换呈现；UI只能绘制获准FrameKey。旧帧不得被标成新来源；可呈现不等于物理屏幕已经显示。
5. 注册／纹理／待提交回调／GPU完成租约由宿主管理，最后实际引用和提交结束后才回收。逻辑注册删除不代表GPU、驱动或私有提交归零。

当前普通呈现共享eframe的Device／Queue，无每帧CPU回读。CPU／GPU合成使用同一Plan语义，直接回读是显式验证入口。1024²等能力上限不能当成stable文件规则；导出尚未实现，须复用语义而非另造一套合成。

## 选择与交互

Workspace使未加入持久ComponentTree的文档也能参与编辑投影；临时共同框不创建组件。图层／组边框由局部几何和明确映射生成，独立于游戏判定区。

UI连续手势保留最后绝对指针／修饰状态，当前来源成为可呈现帧后再替换草稿，避免不停取消在途准备；松手计算最终结果并一次提交。Esc、失焦／视口变化、目标失效和故障立即取消，不等候新帧。暂停只冻结游戏，仍允许编辑／素材更新。

## 故障与重建

普通启动遇致命图形故障冻结并诊断，**没有自动重开窗口**；用户已接受该限制。下述完整窗口保留／重建仅在显式受控验证入口中实现，不能描述为普通产品自动恢复。

故障先关闭图形／声音准入、冻结并取消本UI拥有的草稿，清除手势及迟到松开；外部草稿不能冒名取消。音频单独故障也先重选取消后的实际来源，再接纳图形准备。取消不创建历史节点，真实取消回执增长与项目／历史不变分别检查。

退出保留原ProjectSession，不通过快照复制或补偿Command重建历史。初始化Attempt在全部前置条件成功前持有交接，失败返回同一所有者；FrameKey游标和设备代次不得回退。

运行时任务／音频／绘制租约清理完成，和框架Renderer／surface／整个Instance退休，是两个独立门槛。Pending保留句柄、端点和所有者，可继续核对真实完成；Terminal保持封锁，不因反复重试变成功。只有同一所有者完成端点finish／实际释放后才允许下一后端，不能先退款或提前丢弃失败返回值。

受控恢复保持背景最终复制锚点和同一来源，恢复后仍暂停，显式播放不补旧输入／声音。复制锚点不是设备消费位置。真实TDR／OOM、永久原生异常、其他平台和声学结果仍未实测，不用受控门闩替代。

具体用例和按构建范围见[P2最终验收](P2_FINAL_ACCEPTANCE.md)、[活动编辑故障](P2_ACTIVE_EDIT_FAULTS.md)、[初始化失败](P2_INIT_RECOVERY.md)及[端点归还](P2_ENDPOINT_RETURN.md)。
