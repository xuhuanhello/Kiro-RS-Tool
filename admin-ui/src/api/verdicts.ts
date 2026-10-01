import axios from 'axios'
import { storage } from '@/lib/storage'
import type { ClassifierVerdictPage, ClassifierVerdictQuery } from '@/types/api'

const api = axios.create({
  baseURL: '/api/admin',
  timeout: 15000,
  headers: { 'Content-Type': 'application/json' },
})

api.interceptors.request.use((config) => {
  const apiKey = storage.getApiKey()
  if (apiKey) config.headers['x-api-key'] = apiKey
  return config
})

/** 分页查询服务端分类器裁决（默认按时间倒序，由后端决定） */
export async function getClassifierVerdicts(
  query: ClassifierVerdictQuery,
): Promise<ClassifierVerdictPage> {
  const params: Record<string, string> = {}
  if (query.limit != null) params.limit = String(query.limit)
  if (query.offset != null) params.offset = String(query.offset)
  if (query.onlyFlagged) params.onlyFlagged = 'true'
  if (query.search) params.search = query.search
  const { data } = await api.get<ClassifierVerdictPage>('/classifier-verdicts', {
    params,
  })
  return data
}
