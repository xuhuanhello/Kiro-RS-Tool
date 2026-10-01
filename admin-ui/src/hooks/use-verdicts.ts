import { keepPreviousData, useQuery } from '@tanstack/react-query'
import { getClassifierVerdicts } from '@/api/verdicts'
import type { ClassifierVerdictQuery } from '@/types/api'

/**
 * 分类器裁决查询 hook
 *
 * 裁决随 Claude Code auto 模式实时产生，页面要"接近实时"：保持 5s 短轮询，
 * 并在窗口重新聚焦 / 网络恢复时立即重拉。翻页或改筛选条件时用
 * keepPreviousData 保留上一批结果，避免表格闪成空白。
 */
export function useVerdicts(query: ClassifierVerdictQuery) {
  return useQuery({
    queryKey: ['classifier-verdicts', query],
    queryFn: () => getClassifierVerdicts(query),
    refetchInterval: 5_000,
    staleTime: 1_000,
    placeholderData: keepPreviousData,
    refetchOnMount: 'always',
    refetchOnReconnect: true,
    refetchOnWindowFocus: true,
  })
}
