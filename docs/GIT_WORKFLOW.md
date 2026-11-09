# Git 与远端管理

2026-11-09正式成本轮前备份：上一轮11文件审阅修订经正常完整hook提交并推送76810dde6bab5be73707cba2a1d3c40cf3598c01，PUBLIC origin及远端SHA一致。496项Python／169项gameplay／17项工具Rust、核心与桌面默认／全feature检查通过。3945bb6的Actions 42539581458三项均已成功；[76810dd的Actions 42544000658](https://github.com/sumiyuki727/skin-editor/actions/runs/42544000658)核心Windows／Linux通过，桌面仍在运行。本轮[正式成本及退出记录](K1_EXIT_REVIEW.md#2026-11-09完整成本新轮与退出决定)属于此备份之后的未提交文档；原始测量与冻结程序在忽略目录，不纳入Git。

2026-11-09完整计划增量备份：21个已审查路径经正常完整hook提交为3945bb6483d05dab3f9bf84613335cd79ef11e5b，已推送PUBLIC origin并核对完整SHA；493项Python／169项gameplay／16项工具Rust及核心、桌面默认／全feature检查通过。既有schema.rs规范化空差异未暂存。新增[一致性审阅](K1_COST_CLOSURE_SPEC.md#2026-11-09完整计划一致性审阅)属于此备份之后的未提交修订；[Actions 42539581458](https://github.com/sumiyuki727/skin-editor/actions/runs/42539581458)的Windows／Linux核心检查已通过，桌面任务仍在运行，远端结果仅覆盖备份提交。

2026-11-09当前可见性：用户明确要求PUBLIC，已通过GitHub确认归属sumiyuki727、管理员权限及修改后的PUBLIC状态。轮前检查已扫描远端可达历史1706个对象／1313个blob的常见敏感模式，核对5个自制P0音频fixture来源；既有backup-20261102-8cb882e预发布附件与本机保管清单、ZIP内26项及远端摘要相符，内容为自有源码／文档／锁文件和P2冻结程序。扫描不是完整安全或授权审计，不新增许可证声明。历史PRIVATE记录保留当时事实，今后同步沿用用户的PUBLIC授权，仍需逐轮核对目标及敏感信息。

本轮全面检查文档11文件经正常完整hook提交为42e3b4d28e145cd34f5afd7c5123c76e2349cf3d并推送；远端SHA一致。上个6ac5fda的[Actions 42529617660](https://github.com/sumiyuki727/skin-editor/actions/runs/42529617660)两个核心任务通过，桌面任务因25分钟上限被取消，日志未见测试失败；本轮新备份[Actions 42534918264](https://github.com/sumiyuki727/skin-editor/actions/runs/42534918264)三项全部成功，Windows桌面于2026-11-08T23:44:35Z完成，未重现上一提交超时；完整细节见[本轮实现记录](K1_COST_CLOSURE_SPEC.md#2026-11-09完整计划实现记录)。远端验证仅覆盖备份提交，不覆盖本轮未上传实现。

2026-11-09后续全面检查：上一轮CI测试替身修正与两份说明已通过正常完整hook保存为`6ac5fda1efc342ae72001f60058c4205def4c5b2`，推送PRIVATE origin并核对远端SHA。本轮新文档与GitHub实际复验结果统一见[全面检查](PROJECT_REVIEW_20261109.md#本轮验证与证据身份)。下面故障记录按当时状态保留，不能将其“未提交”理解为修正仍未保存。

2026-11-09 CI故障确认：[019a296的Actions 42523539838](https://github.com/sumiyuki727/skin-editor/actions/runs/42523539838)中Linux核心和Windows桌面通过，Windows核心在Python 3.12.10的test_reparse_points_are_refused_without_following_target失败。硬链接测试以tuple重建stat_result，丢失Windows专用st_file_attributes；新pathlib的lstat委托stat后取得None，触发None与0x400位运算异常。本机Python 3.9.6用同一委托路径成功复现。修正仅用转发替身覆盖st_nlink，保留全部原生属性，并显式覆盖委托路径；生产安全检查、workflow及产品行为均不变。上一轮17文件修订已通过正常完整hook备份推送856cabd5ae34af686bc10f933d6d8aeec63590ea，远端SHA一致；该备份尚不包含本次CI修正，不能声明远端已恢复。本次测试修正与此记录未提交，修正后62项成本工具Python回归及仓库候选／git diff --check通过；直接Python 3.12复验尚缺，日志见本机artifacts/github-query-ci-review/，后续需提交后由远端重新验证。

2026-11-02。远端为 [sumiyuki727/skin-editor](https://github.com/sumiyuki727/skin-editor)，创建时核对为 PRIVATE；当前默认分支沿用 `codex/p1-bootstrap`，不重写已有历史。GitHub 是远端备份与检查入口，本地 Git 仍提供工作区和回退能力。

每轮先读状态、最近提交与暂存差异。已有可审查增量通过仓库 hook 后提交，核对实际提交和用户指定可见性的目标仓库（当前PUBLIC），再正常推送 origin；无变更不造空提交。新增工作在检查前保持未提交，不用绕过 hook、强推、amend 或历史重写换取备份。用户明确要求暂不提交／推送时遵照该轮要求。凭据仅由系统凭据管理器／GitHub CLI 获取，不写入 URL、脚本、文档或环境样例。

2026-11-03回退点：P3-1C完整文件基础经正常hook检查后提交为 `bd3b6262d2a39830d3c94c4c81227cb5725a1700`，已正常推送PRIVATE origin并核对远端SHA。本轮热日志／进程中断恢复另作未提交增量；不将既有远端CI结论套用到本轮未上传代码。

源码、锁文件、自制可重现样例及审阅证据属于 Git。原有 `.github/workflows/core.yml` 在推送后运行核心 Windows／Linux 检查及 Windows 无窗口桌面检查；远端 CI 成败单独记录，不能用本地通过代替。涉及真实设备、窗口、稳定端及平台未覆盖的验证仍按原验收范围管理。

原始诊断、ETL、私人资源、工程、构建产物与工具缓存继续受 `.gitignore` 排除。确有留存必要且来源明确、适合公开的自有产物才可作为当前公开仓库的预发布附件备份，附路径、长度、SHA-256、所属源码提交和验证状态；重新下载核对后才可清理其本地副本。附件不是正式产品发布，也不将尚未完成检查的源码备份冒充 Commit。

空间整理优先处理可重建的 `target`／`incremental` 缓存。递归删除前须解析绝对路径，确认处于明确构建目录，核对无受跟踪文件和重解析点、无活动构建；不删除历史证据、冻结程序、工具链、用户项目或素材。清理只保留清单与释放字节数，不能从旧证据中抹去历史失败。发布附件及 Git 上传不会自动减少本地空间。

本次起点 `8cb882e1389f955f95b6059431fd0c7025adad9e` 已上传并通过远端分支 SHA 核对。P3-1C 未完成检查的修改另行保留，未混入该 Commit。

迁移首轮 CI（Actions 41454081158）发现两项环境差异：Linux 中测试替身需允许创建不存在的 WinDLL 属性，Windows 离线 Cargo metadata 检查需要预取锁文件的跨目标依赖。修正保持原检查范围；首次失败记录继续保留，不追认其通过。

用户授权的恢复附件见 [backup-20261102-8cb882e](https://github.com/sumiyuki727/skin-editor/releases/tag/backup-20261102-8cb882e)，压缩包 7,068,705 字节，SHA-256 `79e41ba81472e75ed28d9f72824815acd5cdd83cc48e0ebeeb99553645ec6889`；已回下载校验全部 25 个文件。该附件的 WIP 源码不等同于后续已验证源码。

首次只清理列明的无受跟踪文件、无重解析点构建缓存：桌面 debug/incremental 12.49 GiB，以及四个旧 P0 实验 target 共约 6.30 GiB。执行清单留在本机 artifacts/github-backup-20261102/cache-cleanup-completed.json；未删除诊断、工程或原始证据，清理后编译产生的新缓存另计。

管理提交 `1778c378e9dd4662a4a1e2cd39473f905b4330a6` 已正常通过仓库hook并推送，独立保留P3-1C工作区增量。其检查结束后，再核对无活动构建／编辑器、无受跟踪文件／重解析点／潜在用户素材，清理 desktop/target/debug 约23.99 GiB（其中包含本轮重新生成的缓存，不能与前次删除量相加冒充净释放）。E盘实际空闲由约27.36增至56.92 GiB。第二次清单为本机 artifacts/github-backup-20261102/final-cache-completed.json；release输出和冻结程序保留，下次桌面调试编译需重新构建。

修复后 [CI 41462169265](https://github.com/sumiyuki727/skin-editor/actions/runs/41462169265) 的Windows核心、Linux核心及Windows桌面三项均通过。它验证管理提交1778c37及其P3-1B基线，不包含本轮尚未提交的P3-1C；当前存储代码另由本机检查证明，不把远端结果套用到未上传源码。
