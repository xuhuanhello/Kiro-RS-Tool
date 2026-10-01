import { useEffect, useState } from 'react'
import {
  ShieldCheck,
  ShieldAlert,
  Zap,
  Search,
  RefreshCw,
  ChevronLeft,
  ChevronRight,
} from 'lucide-react'
import { Card, CardContent } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Input } from '@/components/ui/input'
import { Switch } from '@/components/ui/switch'
import { useVerdicts } from '@/hooks/use-verdicts'
import { extractErrorMessage } from '@/lib/utils'
import type { ClassifierVerdict, ClassifierVerdictQuery } from '@/types/api'

/** 输入防抖：停止输入 delay 毫秒后才把值同步给下游（搜索走服务端） */
function useDebounced<T>(value: T, delay = 300): T {
  const [debounced, setDebounced] = useState(value)
  useEffect(() => {
    const timer = setTimeout(() => setDebounced(value), delay)
    return () => clearTimeout(timer)
  }, [value, delay])
  return debounced
}

function formatTime(ts: string): string {
  const d = new Date(ts)
  if (isNaN(d.getTime())) return ts
  return d.toLocaleString('zh-CN', { hour12: false })
}

function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`
  return `${(ms / 1000).toFixed(2)}s`
}

/** 裁决徽章：红=拦截，绿=放行 */
function VerdictBadge({ flagged }: { flagged: boolean }) {
  if (flagged)
    return (
      <Badge variant="destructive">
        <ShieldAlert className="mr-1 h-3 w-3" />
        拦截
      </Badge>
    )
  return (
    <Badge variant="success">
      <ShieldCheck className="mr-1 h-3 w-3" />
      放行
    </Badge>
  )
}

/** 单条裁决行 */
function VerdictRow({ v }: { v: ClassifierVerdict }) {
  // 有解码结果时以它为主文本，原始命令降级为次要说明
  const primary = v.decodedCommand || v.commandPreview
  const secondary = v.decodedCommand ? v.commandPreview : null
  return (
    <tr className="border-b border-border/40 align-top hover:bg-accent/40">
      <td
        className="py-2.5 pr-3 text-[13px] whitespace-nowrap tabular-nums text-muted-foreground"
        title={`trace ${v.traceId}`}
      >
        {formatTime(v.ts)}
      </td>
      <td className="py-2.5 pr-3 text-[13px] whitespace-nowrap">
        <div className="font-medium">{v.toolName}</div>
        {v.platform && (
          <div className="text-[11px] text-muted-foreground">{v.platform}</div>
        )}
      </td>
      <td className="py-2.5 pr-3">
        <div className="max-w-[520px]">
          <div
            className="truncate font-mono text-[12px] text-foreground"
            title={primary}
          >
            {primary || '—'}
          </div>
          {secondary && (
            <div
              className="truncate font-mono text-[11px] text-muted-foreground"
              title={secondary}
            >
              {secondary}
            </div>
          )}
        </div>
      </td>
      <td className="py-2.5 pr-3">
        <VerdictBadge flagged={v.flagged} />
      </td>
      <td className="py-2.5 pr-3 text-[13px] text-muted-foreground">
        <div className="max-w-[320px] truncate" title={v.reason ?? undefined}>
          {v.reason || '—'}
        </div>
      </td>
      <td className="py-2.5 pr-3 text-[13px] whitespace-nowrap tabular-nums text-muted-foreground">
        {v.cached ? '—' : formatDuration(v.durationMs)}
      </td>
      <td className="py-2.5 pr-3">
        {v.cached ? (
          <Badge variant="outline">
            <Zap className="mr-1 h-3 w-3" />
            命中
          </Badge>
        ) : (
          <span className="text-[13px] text-muted-foreground">—</span>
        )}
      </td>
      <td className="py-2.5 pr-3">
        <div
          className="max-w-[240px] truncate font-mono text-[11px] text-muted-foreground"
          title={v.liveCwd ?? undefined}
        >
          {v.liveCwd || '—'}
        </div>
      </td>
    </tr>
  )
}

const PAGE_SIZE = 50

export function VerdictsPage() {
  const [search, setSearch] = useState('')
  const [onlyFlagged, setOnlyFlagged] = useState(false)
  const [page, setPage] = useState(0)

  const debouncedSearch = useDebounced(search, 300)
  const activeSearch = debouncedSearch.trim()

  // 筛选条件变化时回到第一页
  useEffect(() => {
    setPage(0)
  }, [activeSearch, onlyFlagged])

  const query: ClassifierVerdictQuery = {
    limit: PAGE_SIZE,
    offset: page * PAGE_SIZE,
    search: activeSearch || undefined,
    onlyFlagged: onlyFlagged || undefined,
  }
  const { data, isLoading, isFetching, isError, error, refetch } =
    useVerdicts(query)

  const items = data?.items ?? []
  const total = data?.total ?? 0
  const flaggedCount = items.filter((v) => v.flagged).length
  const totalPages = Math.max(1, Math.ceil(total / PAGE_SIZE))
  const filtered = Boolean(activeSearch) || onlyFlagged

  return (
    <div className="space-y-5">
      <div className="space-y-2">
        {/* 工具条 */}
        <div className="flex flex-wrap items-center gap-3">
          <div className="flex items-center gap-2">
            <ShieldCheck className="h-5 w-5 text-muted-foreground" />
            <h2 className="text-lg font-semibold tracking-tight">分类器裁决</h2>
            {total > 0 && <Badge variant="secondary">{total}</Badge>}
          </div>
          <div className="ml-auto flex flex-wrap items-center gap-2">
            <div className="relative">
              <Search className="pointer-events-none absolute left-2.5 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-muted-foreground" />
              <Input
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                placeholder="搜索命令 / 工具 / 理由"
                className="h-8 w-64 rounded-md pl-8 text-[13px]"
              />
            </div>
            <div className="flex h-8 items-center gap-2 rounded-md border border-border/70 px-2.5">
              <span className="text-[13px] text-muted-foreground">只看拦截</span>
              <Switch checked={onlyFlagged} onCheckedChange={setOnlyFlagged} />
            </div>
            <Button
              size="sm"
              variant="outline"
              onClick={() => refetch()}
              disabled={isFetching}
            >
              <RefreshCw
                className={`h-3.5 w-3.5 ${isFetching ? 'animate-spin' : ''}`}
              />
              刷新
            </Button>
          </div>
        </div>
        {/* 统计文案 */}
        <div className="text-[13px] text-muted-foreground">
          最近 <span className="font-medium text-foreground">{items.length}</span>{' '}
          条裁决，其中{' '}
          <span
            className={
              flaggedCount > 0
                ? 'font-medium text-destructive'
                : 'font-medium text-foreground'
            }
          >
            {flaggedCount}
          </span>{' '}
          条拦截
          {isFetching && <span className="ml-2">· 更新中…</span>}
        </div>
      </div>

      <Card>
        <CardContent className="p-0">
          {isLoading ? (
            <div className="p-6 text-sm text-muted-foreground">加载中…</div>
          ) : isError && items.length === 0 ? (
            <div className="p-6 text-sm">
              <div className="font-medium text-destructive">加载失败</div>
              <p className="mt-1 text-muted-foreground">
                {extractErrorMessage(error)}
              </p>
              <Button
                size="sm"
                variant="outline"
                className="mt-3"
                onClick={() => refetch()}
              >
                重试
              </Button>
            </div>
          ) : items.length === 0 ? (
            <div className="p-6 text-sm text-muted-foreground">
              {filtered
                ? '没有符合条件的裁决，试试调整搜索词或关闭「只看拦截」。'
                : '暂无裁决记录。Claude Code auto 模式触发服务端分类器后，每次裁决都会显示在这里。'}
            </div>
          ) : (
            <>
              {isError && (
                <div className="border-b border-destructive/30 bg-destructive/5 px-3 py-2 text-[13px] text-destructive">
                  刷新失败：{extractErrorMessage(error)}（下方为最近一次成功结果）
                </div>
              )}
              <div className="overflow-x-auto">
                <table className="w-full text-left">
                  <thead>
                    <tr className="border-b border-border/60 text-[12px] uppercase tracking-wider text-muted-foreground">
                      <th className="py-2 pl-3 pr-3 font-medium">时间</th>
                      <th className="py-2 pr-3 font-medium">工具</th>
                      <th className="py-2 pr-3 font-medium">命令</th>
                      <th className="py-2 pr-3 font-medium">裁决</th>
                      <th className="py-2 pr-3 font-medium">理由</th>
                      <th className="py-2 pr-3 font-medium">耗时</th>
                      <th className="py-2 pr-3 font-medium">缓存</th>
                      <th className="py-2 pr-3 font-medium">工作目录</th>
                    </tr>
                  </thead>
                  <tbody>
                    {items.map((v) => (
                      <VerdictRow key={v.id} v={v} />
                    ))}
                  </tbody>
                </table>
              </div>
            </>
          )}
        </CardContent>
      </Card>

      {total > PAGE_SIZE && (
        <div className="flex items-center justify-center gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={() => setPage((p) => Math.max(0, p - 1))}
            disabled={page === 0 || isFetching}
          >
            <ChevronLeft className="h-3.5 w-3.5" />
            上一页
          </Button>
          <div className="px-3 text-sm tabular-nums text-muted-foreground">
            第 <span className="font-medium text-foreground">{page + 1}</span> /{' '}
            {totalPages} 页
            <span className="mx-1.5 text-muted-foreground/50">·</span>共 {total} 条
          </div>
          <Button
            variant="outline"
            size="sm"
            onClick={() => setPage((p) => Math.min(totalPages - 1, p + 1))}
            disabled={page >= totalPages - 1 || isFetching}
          >
            下一页
            <ChevronRight className="h-3.5 w-3.5" />
          </Button>
        </div>
      )}
    </div>
  )
}
