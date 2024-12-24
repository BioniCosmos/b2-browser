import ky from 'ky'

export async function authorizeAccount(appId: string, appKey: string) {
  return ky(`${defaultAPIBaseURL}/b2api/v3/b2_authorize_account`, {
    headers: { Authorization: `Basic ${btoa(`${appId}:${appKey}`)}` },
  }).json<AccountAuthorization>()
}

export async function listFileNames(
  { apiBaseURL, token }: AccountInfo,
  bucketId: string,
  startFileName?: string,
): Promise<FileNameList> {
  const searchParams = startFileName
    ? { bucketId, startFileName }
    : ({ bucketId } as { bucketId: string })
  return ky(`${apiBaseURL}/b2api/v3/b2_list_file_names`, {
    searchParams,
    headers: { Authorization: token },
  }).json<FileNameList>()
}

export interface ErrorDetails {
  status: number
  code: string
  message: string
}

export interface AccountAuthorization {
  accountId: string
  apiInfo: APIInfo
  authorizationToken: string
  applicationKeyExpirationTimestamp: number
}

export interface APIInfo {
  groupsApi: GroupsAPI
  storageApi: StorageAPI
}

export interface GroupsAPI {
  capabilities: string[]
  groupsApiUrl: string
  infoType: string
}

export interface StorageAPI {
  absoluteMinimumPartSize: number
  apiUrl: string
  bucketId: string
  bucketName: string
  capabilities: string[]
  downloadUrl: string
  infoType: string
  namePrefix: string
  recommendedPartSize: number
  s3ApiUrl: string
}

export interface AccountInfo {
  apiBaseURL: string
  token: string
}

export interface FileNameList {
  files: File[]
  nextFileName: string
}

export interface File {
  accountId: string
  action: string
  bucketId: string
  contentLength: number
  contentSha1: string
  contentMd5: string
  contentType: string
  fileId: string
  fileInfo: Record<string, string>
  fileName: string
  fileRetention: AuthorizationField<FileRetention>
  legalHold: AuthorizationField<string>
  serverSideEncryption: ServerSideEncryption
  uploadTimestamp: number
}

export interface AuthorizationField<T> {
  isClientAuthorizedToRead: boolean
  value: T
}

export interface FileRetention {
  mode: string
  retainUntilTimestamp: number
}

export interface ServerSideEncryption {
  algorithm: string
  mode: string
}

const defaultAPIBaseURL = 'https://api.backblazeb2.com'
