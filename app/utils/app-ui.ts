export type AppUiSection
  = | 'activity'
    | 'drag'
    | 'gif'
    | 'job'
    | 'media'
    | 'metrics'
    | 'modes'
    | 'monitor'
    | 'output'
    | 'presets'
    | 'queue'
    | 'resource'
    | 'status'
    | 'toolbar'
    | 'toast'
    | 'updates'

export type AppUiCopy = Record<AppUiSection, Record<string, string>>

export type AppUiInjection = {
  value: AppUiCopy
}
