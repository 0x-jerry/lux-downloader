import { reactive, type InjectionKey, inject } from 'vue'
import type { Task } from '../types'
import type { TaskPatchRequest } from '../../../shared'

export type TaskCommand = 'pause' | 'resume' | 'restart' | 'remove'

export function useTasks() {
  const state = reactive({
    tasks: [] as Task[],
    status: '',
  })

  async function loadTasks() {
    try {
      const response = await browser.runtime.sendMessage({ action: 'list_tasks' })
      if (!response?.ok) {
        state.status = response?.error ?? 'Failed to load tasks'
        return
      }
      state.tasks = (response.data?.items ?? []) as Task[]
      state.status = `Loaded ${state.tasks.length} tasks`
    } catch (error) {
      state.status = error instanceof Error ? error.message : String(error)
    }
  }

  async function createTask(
    url: string,
    referer: string,
    opts?: { destinationPath?: string; overwrite?: boolean },
  ) {
    const response = await browser.runtime.sendMessage({
      action: 'manual_add_task',
      payload: { url, referer, ...opts },
    })

    if (!response?.ok) {
      return { ok: false as const, error: response?.error ?? 'Failed to create task' }
    }

    await loadTasks()
    return { ok: true as const, id: response.data.id as string }
  }

  async function runAction(id: string, command: TaskCommand, deleteFile?: boolean) {
    const response = await browser.runtime.sendMessage({
      action: 'task_action',
      payload: { id, action: command, deleteFile },
    })

    if (!response?.ok) {
      state.status = response?.error ?? `Failed to ${command}`
      return false
    }

    state.status = `${command} succeeded`
    await loadTasks()
    return true
  }

  async function patchTask(id: string, patch: TaskPatchRequest) {
    const response = await browser.runtime.sendMessage({
      action: 'patch_task',
      payload: { id, patch },
    })

    if (!response?.ok) {
      state.status = response?.error ?? 'Failed to update task'
      return false
    }

    state.status = 'Task source updated'
    await loadTasks()
    return true
  }

  function taskTitle(taskId: string | null): string {
    if (!taskId) return ''
    const task = state.tasks.find((t) => t.id === taskId)
    return task?.spec.destination_path || taskId
  }

  return { state, loadTasks, createTask, runAction, patchTask, taskTitle }
}

export type TasksService = ReturnType<typeof useTasks>
export const TASKS_KEY: InjectionKey<TasksService> = Symbol('tasks')

export function useInjectTasks() {
  const service = inject(TASKS_KEY)
  if (!service) throw new Error('Tasks service not provided')
  return service
}
