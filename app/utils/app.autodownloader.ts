export function fileNameFromUrl(url: string) {
  try {
    return decodeURIComponent(new URL(url).pathname.split('/').pop() || 'xcompressor')
  } catch {
    return 'xcompressor'
  }
}
