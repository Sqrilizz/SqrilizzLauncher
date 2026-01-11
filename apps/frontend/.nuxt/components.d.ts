
import type { DefineComponent, SlotsType } from 'vue'
type IslandComponent<T extends DefineComponent> = T & DefineComponent<{}, {refresh: () => Promise<void>}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, SlotsType<{ fallback: { error: unknown } }>>
type HydrationStrategies = {
  hydrateOnVisible?: IntersectionObserverInit | true
  hydrateOnIdle?: number | true
  hydrateOnInteraction?: keyof HTMLElementEventMap | Array<keyof HTMLElementEventMap> | true
  hydrateOnMediaQuery?: string
  hydrateAfter?: number
  hydrateWhen?: boolean
  hydrateNever?: true
}
type LazyComponent<T> = (T & DefineComponent<HydrationStrategies, {}, {}, {}, {}, {}, {}, { hydrated: () => void }>)
interface _GlobalComponents {
      'BrandLogoAnimated': typeof import("../src/components/brand/LogoAnimated.vue")['default']
    'BrandTextLogo': typeof import("../src/components/brand/TextLogo.vue")['default']
    'UiAccordion': typeof import("../src/components/ui/Accordion.vue")['default']
    'UiAdPlaceholder': typeof import("../src/components/ui/AdPlaceholder.vue")['default']
    'UiAutomaticAccordion': typeof import("../src/components/ui/AutomaticAccordion.vue")['default']
    'UiBreadcrumbs': typeof import("../src/components/ui/Breadcrumbs.vue")['default']
    'UiChips': typeof import("../src/components/ui/Chips.vue")['default']
    'UiConditionalNuxtLink': typeof import("../src/components/ui/ConditionalNuxtLink.vue")['default']
    'UiDoubleIcon': typeof import("../src/components/ui/DoubleIcon.vue")['default']
    'UiEnvironmentIndicator': typeof import("../src/components/ui/EnvironmentIndicator.vue")['default']
    'UiFileInput': typeof import("../src/components/ui/FileInput.vue")['default']
    'UiHCaptcha': typeof import("../src/components/ui/HCaptcha.vue")['default']
    'UiMessageBanner': typeof import("../src/components/ui/MessageBanner.vue")['default']
    'UiModal': typeof import("../src/components/ui/Modal.vue")['default']
    'UiNavRow': typeof import("../src/components/ui/NavRow.vue")['default']
    'UiNavStack': typeof import("../src/components/ui/NavStack.vue")['default']
    'UiNavTabs': typeof import("../src/components/ui/NavTabs.vue")['default']
    'UiNewsletterButton': typeof import("../src/components/ui/NewsletterButton.vue")['default']
    'UiNotificationItem': typeof import("../src/components/ui/NotificationItem.vue")['default']
    'UiOptionGroup': typeof import("../src/components/ui/OptionGroup.vue")['default']
    'UiOrganizationProjectTransferModal': typeof import("../src/components/ui/OrganizationProjectTransferModal.vue")['default']
    'UiProjectCard': typeof import("../src/components/ui/ProjectCard.vue")['default']
    'UiProjectMemberHeader': typeof import("../src/components/ui/ProjectMemberHeader.vue")['default']
    'UiShareArticleButtons': typeof import("../src/components/ui/ShareArticleButtons.vue")['default']
    'UiAdminBatchCreditModal': typeof import("../src/components/ui/admin/BatchCreditModal.vue")['default']
    'UiChartsChart': typeof import("../src/components/ui/charts/Chart.client.vue")['default']
    'UiChartsChartDisplay': typeof import("../src/components/ui/charts/ChartDisplay.vue")['default']
    'UiChartsCompactChart': typeof import("../src/components/ui/charts/CompactChart.client.vue")['default']
    'UiCreateCollectionCreateModal': typeof import("../src/components/ui/create/CollectionCreateModal.vue")['default']
    'UiCreateLimitAlert': typeof import("../src/components/ui/create/CreateLimitAlert.vue")['default']
    'UiCreateOrganizationCreateModal': typeof import("../src/components/ui/create/OrganizationCreateModal.vue")['default']
    'UiCreateProjectCreateModal': typeof import("../src/components/ui/create/ProjectCreateModal.vue")['default']
    'UiDashboardCreatorTaxFormModal': typeof import("../src/components/ui/dashboard/CreatorTaxFormModal.vue")['default']
    'UiDashboardCreatorWithdrawModal': typeof import("../src/components/ui/dashboard/CreatorWithdrawModal.vue")['default']
    'UiDashboardRevenueInputField': typeof import("../src/components/ui/dashboard/RevenueInputField.vue")['default']
    'UiDashboardRevenueTransaction': typeof import("../src/components/ui/dashboard/RevenueTransaction.vue")['default']
    'UiDashboardWithdrawFeeBreakdown': typeof import("../src/components/ui/dashboard/WithdrawFeeBreakdown.vue")['default']
    'UiDashboardWithdrawStagesCompletionStage': typeof import("../src/components/ui/dashboard/withdraw-stages/CompletionStage.vue")['default']
    'UiDashboardWithdrawStagesLegacyPaypalDetailsStage': typeof import("../src/components/ui/dashboard/withdraw-stages/LegacyPaypalDetailsStage.vue")['default']
    'UiDashboardWithdrawStagesMethodSelectionStage': typeof import("../src/components/ui/dashboard/withdraw-stages/MethodSelectionStage.vue")['default']
    'UiDashboardWithdrawStagesMuralpayDetailsStage': typeof import("../src/components/ui/dashboard/withdraw-stages/MuralpayDetailsStage.vue")['default']
    'UiDashboardWithdrawStagesMuralpayKycStage': typeof import("../src/components/ui/dashboard/withdraw-stages/MuralpayKycStage.vue")['default']
    'UiDashboardWithdrawStagesTaxFormStage': typeof import("../src/components/ui/dashboard/withdraw-stages/TaxFormStage.vue")['default']
    'UiDashboardWithdrawStagesTremendousDetailsStage': typeof import("../src/components/ui/dashboard/withdraw-stages/TremendousDetailsStage.vue")['default']
    'UiModerationDelphiReportCard': typeof import("../src/components/ui/moderation/ModerationDelphiReportCard.vue")['default']
    'UiModerationProjectNags': typeof import("../src/components/ui/moderation/ModerationProjectNags.vue")['default']
    'UiModerationQueueCard': typeof import("../src/components/ui/moderation/ModerationQueueCard.vue")['default']
    'UiModerationReportCard': typeof import("../src/components/ui/moderation/ModerationReportCard.vue")['default']
    'UiModerationChecklistKeybindsModal': typeof import("../src/components/ui/moderation/checklist/ChecklistKeybindsModal.vue")['default']
    'UiModerationChecklist': typeof import("../src/components/ui/moderation/checklist/ModerationChecklist.vue")['default']
    'UiModerationChecklistModpackPermissionsFlow': typeof import("../src/components/ui/moderation/checklist/ModpackPermissionsFlow.vue")['default']
    'UiModrinthLoadingIndicator': typeof import("../src/components/ui/modrinth-loading-indicator")['default']
    'UiNewsLatestNewsRow': typeof import("../src/components/ui/news/LatestNewsRow.vue")['default']
    'UiReportInfo': typeof import("../src/components/ui/report/ReportInfo.vue")['default']
    'UiReportView': typeof import("../src/components/ui/report/ReportView.vue")['default']
    'UiReportReportsList': typeof import("../src/components/ui/report/ReportsList.vue")['default']
    'UiSearchCategories': typeof import("../src/components/ui/search/Categories.vue")['default']
    'UiServersBackupCreateModal': typeof import("../src/components/ui/servers/BackupCreateModal.vue")['default']
    'UiServersBackupDeleteModal': typeof import("../src/components/ui/servers/BackupDeleteModal.vue")['default']
    'UiServersBackupItem': typeof import("../src/components/ui/servers/BackupItem.vue")['default']
    'UiServersBackupRenameModal': typeof import("../src/components/ui/servers/BackupRenameModal.vue")['default']
    'UiServersBackupRestoreModal': typeof import("../src/components/ui/servers/BackupRestoreModal.vue")['default']
    'UiServersBackupSettingsModal': typeof import("../src/components/ui/servers/BackupSettingsModal.vue")['default']
    'UiServersContentVersionEditModal': typeof import("../src/components/ui/servers/ContentVersionEditModal.vue")['default']
    'UiServersContentVersionFilter': typeof import("../src/components/ui/servers/ContentVersionFilter.vue")['default']
    'UiServersFileItem': typeof import("../src/components/ui/servers/FileItem.vue")['default']
    'UiServersFileManagerError': typeof import("../src/components/ui/servers/FileManagerError.vue")['default']
    'UiServersFileVirtualList': typeof import("../src/components/ui/servers/FileVirtualList.vue")['default']
    'UiServersFilesBrowseNavbar': typeof import("../src/components/ui/servers/FilesBrowseNavbar.vue")['default']
    'UiServersFilesContextMenu': typeof import("../src/components/ui/servers/FilesContextMenu.vue")['default']
    'UiServersFilesCreateItemModal': typeof import("../src/components/ui/servers/FilesCreateItemModal.vue")['default']
    'UiServersFilesDeleteItemModal': typeof import("../src/components/ui/servers/FilesDeleteItemModal.vue")['default']
    'UiServersFilesEditingNavbar': typeof import("../src/components/ui/servers/FilesEditingNavbar.vue")['default']
    'UiServersFilesImageViewer': typeof import("../src/components/ui/servers/FilesImageViewer.vue")['default']
    'UiServersFilesLabelBar': typeof import("../src/components/ui/servers/FilesLabelBar.vue")['default']
    'UiServersFilesMoveItemModal': typeof import("../src/components/ui/servers/FilesMoveItemModal.vue")['default']
    'UiServersFilesRenameItemModal': typeof import("../src/components/ui/servers/FilesRenameItemModal.vue")['default']
    'UiServersFilesUploadConflictModal': typeof import("../src/components/ui/servers/FilesUploadConflictModal.vue")['default']
    'UiServersFilesUploadDragAndDrop': typeof import("../src/components/ui/servers/FilesUploadDragAndDrop.vue")['default']
    'UiServersFilesUploadDropdown': typeof import("../src/components/ui/servers/FilesUploadDropdown.vue")['default']
    'UiServersFilesUploadZipUrlModal': typeof import("../src/components/ui/servers/FilesUploadZipUrlModal.vue")['default']
    'UiServersGlobe': typeof import("../src/components/ui/servers/Globe.vue")['default']
    'UiServersInstallingTicker': typeof import("../src/components/ui/servers/InstallingTicker.vue")['default']
    'UiServersLoaderSelector': typeof import("../src/components/ui/servers/LoaderSelector.vue")['default']
    'UiServersLoaderSelectorCard': typeof import("../src/components/ui/servers/LoaderSelectorCard.vue")['default']
    'UiServersLogLine': typeof import("../src/components/ui/servers/LogLine.vue")['default']
    'UiServersModrinthServersIcon': typeof import("../src/components/ui/servers/ModrinthServersIcon.vue")['default']
    'UiServersPanelServerActionButton': typeof import("../src/components/ui/servers/PanelServerActionButton.vue")['default']
    'UiServersPanelServerStatus': typeof import("../src/components/ui/servers/PanelServerStatus.vue")['default']
    'UiServersPanelSpinner': typeof import("../src/components/ui/servers/PanelSpinner.vue")['default']
    'UiServersPanelTerminal': typeof import("../src/components/ui/servers/PanelTerminal.vue")['default']
    'UiServersPlatformChangeModpackVersionModal': typeof import("../src/components/ui/servers/PlatformChangeModpackVersionModal.vue")['default']
    'UiServersPlatformMrpackModal': typeof import("../src/components/ui/servers/PlatformMrpackModal.vue")['default']
    'UiServersPlatformVersionSelectModal': typeof import("../src/components/ui/servers/PlatformVersionSelectModal.vue")['default']
    'UiServersSaveBanner': typeof import("../src/components/ui/servers/SaveBanner.vue")['default']
    'UiServersServerInstallation': typeof import("../src/components/ui/servers/ServerInstallation.vue")['default']
    'UiServersServerSidebar': typeof import("../src/components/ui/servers/ServerSidebar.vue")['default']
    'UiServersServerStats': typeof import("../src/components/ui/servers/ServerStats.vue")['default']
    'UiServersUpgradeModalWrapper': typeof import("../src/components/ui/servers/ServersUpgradeModalWrapper.vue")['default']
    'UiServersTeleportOverflowMenu': typeof import("../src/components/ui/servers/TeleportOverflowMenu.vue")['default']
    'UiServersIconsChevronDownIcon': typeof import("../src/components/ui/servers/icons/ChevronDownIcon.vue")['default']
    'UiServersIconsChevronUpIcon': typeof import("../src/components/ui/servers/icons/ChevronUpIcon.vue")['default']
    'UiServersIconsCodeFileIcon': typeof import("../src/components/ui/servers/icons/CodeFileIcon.vue")['default']
    'UiServersIconsCogFolderIcon': typeof import("../src/components/ui/servers/icons/CogFolderIcon.vue")['default']
    'UiServersIconsEarthIcon': typeof import("../src/components/ui/servers/icons/EarthIcon.vue")['default']
    'UiServersIconsFullscreenIcon': typeof import("../src/components/ui/servers/icons/FullscreenIcon.vue")['default']
    'UiServersIconsImageFileIcon': typeof import("../src/components/ui/servers/icons/ImageFileIcon.vue")['default']
    'UiServersIconsLoaderIcon': typeof import("../src/components/ui/servers/icons/LoaderIcon.vue")['default']
    'UiServersIconsLoadingIcon': typeof import("../src/components/ui/servers/icons/LoadingIcon.vue")['default']
    'UiServersIconsMinimizeIconVue': typeof import("../src/components/ui/servers/icons/MinimizeIcon.vue.vue")['default']
    'UiServersIconsPanelErrorIcon': typeof import("../src/components/ui/servers/icons/PanelErrorIcon.vue")['default']
    'UiServersIconsSlashIcon': typeof import("../src/components/ui/servers/icons/SlashIcon.vue")['default']
    'UiServersIconsTextFileIcon': typeof import("../src/components/ui/servers/icons/TextFileIcon.vue")['default']
    'UiServersIconsTimer': typeof import("../src/components/ui/servers/icons/Timer.vue")['default']
    'UiServersMarketingMedalPlanPromotion': typeof import("../src/components/ui/servers/marketing/MedalPlanPromotion.vue")['default']
    'UiServersMarketingMedalServerCountdown': typeof import("../src/components/ui/servers/marketing/MedalServerCountdown.vue")['default']
    'UiServersMarketingServerPlanSelector': typeof import("../src/components/ui/servers/marketing/ServerPlanSelector.vue")['default']
    'UiServersNoticeAssignNoticeModal': typeof import("../src/components/ui/servers/notice/AssignNoticeModal.vue")['default']
    'UiServersNoticeDashboardItem': typeof import("../src/components/ui/servers/notice/NoticeDashboardItem.vue")['default']
    'UiThreadConversationThread': typeof import("../src/components/ui/thread/ConversationThread.vue")['default']
    'UiThreadReportThread': typeof import("../src/components/ui/thread/ReportThread.vue")['default']
    'UiThreadMessage': typeof import("../src/components/ui/thread/ThreadMessage.vue")['default']
    'UiThreadSummary': typeof import("../src/components/ui/thread/ThreadSummary.vue")['default']
    'NuxtWelcome': typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/welcome.vue")['default']
    'NuxtLayout': typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/nuxt-layout")['default']
    'NuxtErrorBoundary': typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/nuxt-error-boundary.vue")['default']
    'ClientOnly': typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/client-only")['default']
    'DevOnly': typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/dev-only")['default']
    'ServerPlaceholder': typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/server-placeholder")['default']
    'NuxtLink': typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/nuxt-link")['default']
    'NuxtLoadingIndicator': typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/nuxt-loading-indicator")['default']
    'NuxtTime': typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/nuxt-time.vue")['default']
    'NuxtRouteAnnouncer': typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/nuxt-route-announcer")['default']
    'NuxtImg': typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/nuxt-stubs")['NuxtImg']
    'NuxtPicture': typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/nuxt-stubs")['NuxtPicture']
    'IntlFormatted': typeof import("@vintl/vintl/components")['IntlFormatted']
    'NuxtPage': typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/pages/runtime/page")['default']
    'NoScript': typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/head/runtime/components")['NoScript']
    'Link': typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/head/runtime/components")['Link']
    'Base': typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/head/runtime/components")['Base']
    'Title': typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/head/runtime/components")['Title']
    'Meta': typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/head/runtime/components")['Meta']
    'Style': typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/head/runtime/components")['Style']
    'Head': typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/head/runtime/components")['Head']
    'Html': typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/head/runtime/components")['Html']
    'Body': typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/head/runtime/components")['Body']
    'NuxtIsland': typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/nuxt-island")['default']
    'UiChartsChart': typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/server-placeholder")['default']
    'UiChartsCompactChart': typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/server-placeholder")['default']
    'NuxtRouteAnnouncer': typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/server-placeholder")['default']
      'LazyBrandLogoAnimated': LazyComponent<typeof import("../src/components/brand/LogoAnimated.vue")['default']>
    'LazyBrandTextLogo': LazyComponent<typeof import("../src/components/brand/TextLogo.vue")['default']>
    'LazyUiAccordion': LazyComponent<typeof import("../src/components/ui/Accordion.vue")['default']>
    'LazyUiAdPlaceholder': LazyComponent<typeof import("../src/components/ui/AdPlaceholder.vue")['default']>
    'LazyUiAutomaticAccordion': LazyComponent<typeof import("../src/components/ui/AutomaticAccordion.vue")['default']>
    'LazyUiBreadcrumbs': LazyComponent<typeof import("../src/components/ui/Breadcrumbs.vue")['default']>
    'LazyUiChips': LazyComponent<typeof import("../src/components/ui/Chips.vue")['default']>
    'LazyUiConditionalNuxtLink': LazyComponent<typeof import("../src/components/ui/ConditionalNuxtLink.vue")['default']>
    'LazyUiDoubleIcon': LazyComponent<typeof import("../src/components/ui/DoubleIcon.vue")['default']>
    'LazyUiEnvironmentIndicator': LazyComponent<typeof import("../src/components/ui/EnvironmentIndicator.vue")['default']>
    'LazyUiFileInput': LazyComponent<typeof import("../src/components/ui/FileInput.vue")['default']>
    'LazyUiHCaptcha': LazyComponent<typeof import("../src/components/ui/HCaptcha.vue")['default']>
    'LazyUiMessageBanner': LazyComponent<typeof import("../src/components/ui/MessageBanner.vue")['default']>
    'LazyUiModal': LazyComponent<typeof import("../src/components/ui/Modal.vue")['default']>
    'LazyUiNavRow': LazyComponent<typeof import("../src/components/ui/NavRow.vue")['default']>
    'LazyUiNavStack': LazyComponent<typeof import("../src/components/ui/NavStack.vue")['default']>
    'LazyUiNavTabs': LazyComponent<typeof import("../src/components/ui/NavTabs.vue")['default']>
    'LazyUiNewsletterButton': LazyComponent<typeof import("../src/components/ui/NewsletterButton.vue")['default']>
    'LazyUiNotificationItem': LazyComponent<typeof import("../src/components/ui/NotificationItem.vue")['default']>
    'LazyUiOptionGroup': LazyComponent<typeof import("../src/components/ui/OptionGroup.vue")['default']>
    'LazyUiOrganizationProjectTransferModal': LazyComponent<typeof import("../src/components/ui/OrganizationProjectTransferModal.vue")['default']>
    'LazyUiProjectCard': LazyComponent<typeof import("../src/components/ui/ProjectCard.vue")['default']>
    'LazyUiProjectMemberHeader': LazyComponent<typeof import("../src/components/ui/ProjectMemberHeader.vue")['default']>
    'LazyUiShareArticleButtons': LazyComponent<typeof import("../src/components/ui/ShareArticleButtons.vue")['default']>
    'LazyUiAdminBatchCreditModal': LazyComponent<typeof import("../src/components/ui/admin/BatchCreditModal.vue")['default']>
    'LazyUiChartsChart': LazyComponent<typeof import("../src/components/ui/charts/Chart.client.vue")['default']>
    'LazyUiChartsChartDisplay': LazyComponent<typeof import("../src/components/ui/charts/ChartDisplay.vue")['default']>
    'LazyUiChartsCompactChart': LazyComponent<typeof import("../src/components/ui/charts/CompactChart.client.vue")['default']>
    'LazyUiCreateCollectionCreateModal': LazyComponent<typeof import("../src/components/ui/create/CollectionCreateModal.vue")['default']>
    'LazyUiCreateLimitAlert': LazyComponent<typeof import("../src/components/ui/create/CreateLimitAlert.vue")['default']>
    'LazyUiCreateOrganizationCreateModal': LazyComponent<typeof import("../src/components/ui/create/OrganizationCreateModal.vue")['default']>
    'LazyUiCreateProjectCreateModal': LazyComponent<typeof import("../src/components/ui/create/ProjectCreateModal.vue")['default']>
    'LazyUiDashboardCreatorTaxFormModal': LazyComponent<typeof import("../src/components/ui/dashboard/CreatorTaxFormModal.vue")['default']>
    'LazyUiDashboardCreatorWithdrawModal': LazyComponent<typeof import("../src/components/ui/dashboard/CreatorWithdrawModal.vue")['default']>
    'LazyUiDashboardRevenueInputField': LazyComponent<typeof import("../src/components/ui/dashboard/RevenueInputField.vue")['default']>
    'LazyUiDashboardRevenueTransaction': LazyComponent<typeof import("../src/components/ui/dashboard/RevenueTransaction.vue")['default']>
    'LazyUiDashboardWithdrawFeeBreakdown': LazyComponent<typeof import("../src/components/ui/dashboard/WithdrawFeeBreakdown.vue")['default']>
    'LazyUiDashboardWithdrawStagesCompletionStage': LazyComponent<typeof import("../src/components/ui/dashboard/withdraw-stages/CompletionStage.vue")['default']>
    'LazyUiDashboardWithdrawStagesLegacyPaypalDetailsStage': LazyComponent<typeof import("../src/components/ui/dashboard/withdraw-stages/LegacyPaypalDetailsStage.vue")['default']>
    'LazyUiDashboardWithdrawStagesMethodSelectionStage': LazyComponent<typeof import("../src/components/ui/dashboard/withdraw-stages/MethodSelectionStage.vue")['default']>
    'LazyUiDashboardWithdrawStagesMuralpayDetailsStage': LazyComponent<typeof import("../src/components/ui/dashboard/withdraw-stages/MuralpayDetailsStage.vue")['default']>
    'LazyUiDashboardWithdrawStagesMuralpayKycStage': LazyComponent<typeof import("../src/components/ui/dashboard/withdraw-stages/MuralpayKycStage.vue")['default']>
    'LazyUiDashboardWithdrawStagesTaxFormStage': LazyComponent<typeof import("../src/components/ui/dashboard/withdraw-stages/TaxFormStage.vue")['default']>
    'LazyUiDashboardWithdrawStagesTremendousDetailsStage': LazyComponent<typeof import("../src/components/ui/dashboard/withdraw-stages/TremendousDetailsStage.vue")['default']>
    'LazyUiModerationDelphiReportCard': LazyComponent<typeof import("../src/components/ui/moderation/ModerationDelphiReportCard.vue")['default']>
    'LazyUiModerationProjectNags': LazyComponent<typeof import("../src/components/ui/moderation/ModerationProjectNags.vue")['default']>
    'LazyUiModerationQueueCard': LazyComponent<typeof import("../src/components/ui/moderation/ModerationQueueCard.vue")['default']>
    'LazyUiModerationReportCard': LazyComponent<typeof import("../src/components/ui/moderation/ModerationReportCard.vue")['default']>
    'LazyUiModerationChecklistKeybindsModal': LazyComponent<typeof import("../src/components/ui/moderation/checklist/ChecklistKeybindsModal.vue")['default']>
    'LazyUiModerationChecklist': LazyComponent<typeof import("../src/components/ui/moderation/checklist/ModerationChecklist.vue")['default']>
    'LazyUiModerationChecklistModpackPermissionsFlow': LazyComponent<typeof import("../src/components/ui/moderation/checklist/ModpackPermissionsFlow.vue")['default']>
    'LazyUiModrinthLoadingIndicator': LazyComponent<typeof import("../src/components/ui/modrinth-loading-indicator")['default']>
    'LazyUiNewsLatestNewsRow': LazyComponent<typeof import("../src/components/ui/news/LatestNewsRow.vue")['default']>
    'LazyUiReportInfo': LazyComponent<typeof import("../src/components/ui/report/ReportInfo.vue")['default']>
    'LazyUiReportView': LazyComponent<typeof import("../src/components/ui/report/ReportView.vue")['default']>
    'LazyUiReportReportsList': LazyComponent<typeof import("../src/components/ui/report/ReportsList.vue")['default']>
    'LazyUiSearchCategories': LazyComponent<typeof import("../src/components/ui/search/Categories.vue")['default']>
    'LazyUiServersBackupCreateModal': LazyComponent<typeof import("../src/components/ui/servers/BackupCreateModal.vue")['default']>
    'LazyUiServersBackupDeleteModal': LazyComponent<typeof import("../src/components/ui/servers/BackupDeleteModal.vue")['default']>
    'LazyUiServersBackupItem': LazyComponent<typeof import("../src/components/ui/servers/BackupItem.vue")['default']>
    'LazyUiServersBackupRenameModal': LazyComponent<typeof import("../src/components/ui/servers/BackupRenameModal.vue")['default']>
    'LazyUiServersBackupRestoreModal': LazyComponent<typeof import("../src/components/ui/servers/BackupRestoreModal.vue")['default']>
    'LazyUiServersBackupSettingsModal': LazyComponent<typeof import("../src/components/ui/servers/BackupSettingsModal.vue")['default']>
    'LazyUiServersContentVersionEditModal': LazyComponent<typeof import("../src/components/ui/servers/ContentVersionEditModal.vue")['default']>
    'LazyUiServersContentVersionFilter': LazyComponent<typeof import("../src/components/ui/servers/ContentVersionFilter.vue")['default']>
    'LazyUiServersFileItem': LazyComponent<typeof import("../src/components/ui/servers/FileItem.vue")['default']>
    'LazyUiServersFileManagerError': LazyComponent<typeof import("../src/components/ui/servers/FileManagerError.vue")['default']>
    'LazyUiServersFileVirtualList': LazyComponent<typeof import("../src/components/ui/servers/FileVirtualList.vue")['default']>
    'LazyUiServersFilesBrowseNavbar': LazyComponent<typeof import("../src/components/ui/servers/FilesBrowseNavbar.vue")['default']>
    'LazyUiServersFilesContextMenu': LazyComponent<typeof import("../src/components/ui/servers/FilesContextMenu.vue")['default']>
    'LazyUiServersFilesCreateItemModal': LazyComponent<typeof import("../src/components/ui/servers/FilesCreateItemModal.vue")['default']>
    'LazyUiServersFilesDeleteItemModal': LazyComponent<typeof import("../src/components/ui/servers/FilesDeleteItemModal.vue")['default']>
    'LazyUiServersFilesEditingNavbar': LazyComponent<typeof import("../src/components/ui/servers/FilesEditingNavbar.vue")['default']>
    'LazyUiServersFilesImageViewer': LazyComponent<typeof import("../src/components/ui/servers/FilesImageViewer.vue")['default']>
    'LazyUiServersFilesLabelBar': LazyComponent<typeof import("../src/components/ui/servers/FilesLabelBar.vue")['default']>
    'LazyUiServersFilesMoveItemModal': LazyComponent<typeof import("../src/components/ui/servers/FilesMoveItemModal.vue")['default']>
    'LazyUiServersFilesRenameItemModal': LazyComponent<typeof import("../src/components/ui/servers/FilesRenameItemModal.vue")['default']>
    'LazyUiServersFilesUploadConflictModal': LazyComponent<typeof import("../src/components/ui/servers/FilesUploadConflictModal.vue")['default']>
    'LazyUiServersFilesUploadDragAndDrop': LazyComponent<typeof import("../src/components/ui/servers/FilesUploadDragAndDrop.vue")['default']>
    'LazyUiServersFilesUploadDropdown': LazyComponent<typeof import("../src/components/ui/servers/FilesUploadDropdown.vue")['default']>
    'LazyUiServersFilesUploadZipUrlModal': LazyComponent<typeof import("../src/components/ui/servers/FilesUploadZipUrlModal.vue")['default']>
    'LazyUiServersGlobe': LazyComponent<typeof import("../src/components/ui/servers/Globe.vue")['default']>
    'LazyUiServersInstallingTicker': LazyComponent<typeof import("../src/components/ui/servers/InstallingTicker.vue")['default']>
    'LazyUiServersLoaderSelector': LazyComponent<typeof import("../src/components/ui/servers/LoaderSelector.vue")['default']>
    'LazyUiServersLoaderSelectorCard': LazyComponent<typeof import("../src/components/ui/servers/LoaderSelectorCard.vue")['default']>
    'LazyUiServersLogLine': LazyComponent<typeof import("../src/components/ui/servers/LogLine.vue")['default']>
    'LazyUiServersModrinthServersIcon': LazyComponent<typeof import("../src/components/ui/servers/ModrinthServersIcon.vue")['default']>
    'LazyUiServersPanelServerActionButton': LazyComponent<typeof import("../src/components/ui/servers/PanelServerActionButton.vue")['default']>
    'LazyUiServersPanelServerStatus': LazyComponent<typeof import("../src/components/ui/servers/PanelServerStatus.vue")['default']>
    'LazyUiServersPanelSpinner': LazyComponent<typeof import("../src/components/ui/servers/PanelSpinner.vue")['default']>
    'LazyUiServersPanelTerminal': LazyComponent<typeof import("../src/components/ui/servers/PanelTerminal.vue")['default']>
    'LazyUiServersPlatformChangeModpackVersionModal': LazyComponent<typeof import("../src/components/ui/servers/PlatformChangeModpackVersionModal.vue")['default']>
    'LazyUiServersPlatformMrpackModal': LazyComponent<typeof import("../src/components/ui/servers/PlatformMrpackModal.vue")['default']>
    'LazyUiServersPlatformVersionSelectModal': LazyComponent<typeof import("../src/components/ui/servers/PlatformVersionSelectModal.vue")['default']>
    'LazyUiServersSaveBanner': LazyComponent<typeof import("../src/components/ui/servers/SaveBanner.vue")['default']>
    'LazyUiServersServerInstallation': LazyComponent<typeof import("../src/components/ui/servers/ServerInstallation.vue")['default']>
    'LazyUiServersServerSidebar': LazyComponent<typeof import("../src/components/ui/servers/ServerSidebar.vue")['default']>
    'LazyUiServersServerStats': LazyComponent<typeof import("../src/components/ui/servers/ServerStats.vue")['default']>
    'LazyUiServersUpgradeModalWrapper': LazyComponent<typeof import("../src/components/ui/servers/ServersUpgradeModalWrapper.vue")['default']>
    'LazyUiServersTeleportOverflowMenu': LazyComponent<typeof import("../src/components/ui/servers/TeleportOverflowMenu.vue")['default']>
    'LazyUiServersIconsChevronDownIcon': LazyComponent<typeof import("../src/components/ui/servers/icons/ChevronDownIcon.vue")['default']>
    'LazyUiServersIconsChevronUpIcon': LazyComponent<typeof import("../src/components/ui/servers/icons/ChevronUpIcon.vue")['default']>
    'LazyUiServersIconsCodeFileIcon': LazyComponent<typeof import("../src/components/ui/servers/icons/CodeFileIcon.vue")['default']>
    'LazyUiServersIconsCogFolderIcon': LazyComponent<typeof import("../src/components/ui/servers/icons/CogFolderIcon.vue")['default']>
    'LazyUiServersIconsEarthIcon': LazyComponent<typeof import("../src/components/ui/servers/icons/EarthIcon.vue")['default']>
    'LazyUiServersIconsFullscreenIcon': LazyComponent<typeof import("../src/components/ui/servers/icons/FullscreenIcon.vue")['default']>
    'LazyUiServersIconsImageFileIcon': LazyComponent<typeof import("../src/components/ui/servers/icons/ImageFileIcon.vue")['default']>
    'LazyUiServersIconsLoaderIcon': LazyComponent<typeof import("../src/components/ui/servers/icons/LoaderIcon.vue")['default']>
    'LazyUiServersIconsLoadingIcon': LazyComponent<typeof import("../src/components/ui/servers/icons/LoadingIcon.vue")['default']>
    'LazyUiServersIconsMinimizeIconVue': LazyComponent<typeof import("../src/components/ui/servers/icons/MinimizeIcon.vue.vue")['default']>
    'LazyUiServersIconsPanelErrorIcon': LazyComponent<typeof import("../src/components/ui/servers/icons/PanelErrorIcon.vue")['default']>
    'LazyUiServersIconsSlashIcon': LazyComponent<typeof import("../src/components/ui/servers/icons/SlashIcon.vue")['default']>
    'LazyUiServersIconsTextFileIcon': LazyComponent<typeof import("../src/components/ui/servers/icons/TextFileIcon.vue")['default']>
    'LazyUiServersIconsTimer': LazyComponent<typeof import("../src/components/ui/servers/icons/Timer.vue")['default']>
    'LazyUiServersMarketingMedalPlanPromotion': LazyComponent<typeof import("../src/components/ui/servers/marketing/MedalPlanPromotion.vue")['default']>
    'LazyUiServersMarketingMedalServerCountdown': LazyComponent<typeof import("../src/components/ui/servers/marketing/MedalServerCountdown.vue")['default']>
    'LazyUiServersMarketingServerPlanSelector': LazyComponent<typeof import("../src/components/ui/servers/marketing/ServerPlanSelector.vue")['default']>
    'LazyUiServersNoticeAssignNoticeModal': LazyComponent<typeof import("../src/components/ui/servers/notice/AssignNoticeModal.vue")['default']>
    'LazyUiServersNoticeDashboardItem': LazyComponent<typeof import("../src/components/ui/servers/notice/NoticeDashboardItem.vue")['default']>
    'LazyUiThreadConversationThread': LazyComponent<typeof import("../src/components/ui/thread/ConversationThread.vue")['default']>
    'LazyUiThreadReportThread': LazyComponent<typeof import("../src/components/ui/thread/ReportThread.vue")['default']>
    'LazyUiThreadMessage': LazyComponent<typeof import("../src/components/ui/thread/ThreadMessage.vue")['default']>
    'LazyUiThreadSummary': LazyComponent<typeof import("../src/components/ui/thread/ThreadSummary.vue")['default']>
    'LazyNuxtWelcome': LazyComponent<typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/welcome.vue")['default']>
    'LazyNuxtLayout': LazyComponent<typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/nuxt-layout")['default']>
    'LazyNuxtErrorBoundary': LazyComponent<typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/nuxt-error-boundary.vue")['default']>
    'LazyClientOnly': LazyComponent<typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/client-only")['default']>
    'LazyDevOnly': LazyComponent<typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/dev-only")['default']>
    'LazyServerPlaceholder': LazyComponent<typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/server-placeholder")['default']>
    'LazyNuxtLink': LazyComponent<typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/nuxt-link")['default']>
    'LazyNuxtLoadingIndicator': LazyComponent<typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/nuxt-loading-indicator")['default']>
    'LazyNuxtTime': LazyComponent<typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/nuxt-time.vue")['default']>
    'LazyNuxtRouteAnnouncer': LazyComponent<typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/nuxt-route-announcer")['default']>
    'LazyNuxtImg': LazyComponent<typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/nuxt-stubs")['NuxtImg']>
    'LazyNuxtPicture': LazyComponent<typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/nuxt-stubs")['NuxtPicture']>
    'LazyIntlFormatted': LazyComponent<typeof import("@vintl/vintl/components")['IntlFormatted']>
    'LazyNuxtPage': LazyComponent<typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/pages/runtime/page")['default']>
    'LazyNoScript': LazyComponent<typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/head/runtime/components")['NoScript']>
    'LazyLink': LazyComponent<typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/head/runtime/components")['Link']>
    'LazyBase': LazyComponent<typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/head/runtime/components")['Base']>
    'LazyTitle': LazyComponent<typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/head/runtime/components")['Title']>
    'LazyMeta': LazyComponent<typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/head/runtime/components")['Meta']>
    'LazyStyle': LazyComponent<typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/head/runtime/components")['Style']>
    'LazyHead': LazyComponent<typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/head/runtime/components")['Head']>
    'LazyHtml': LazyComponent<typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/head/runtime/components")['Html']>
    'LazyBody': LazyComponent<typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/head/runtime/components")['Body']>
    'LazyNuxtIsland': LazyComponent<typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/nuxt-island")['default']>
    'LazyUiChartsChart': LazyComponent<typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/server-placeholder")['default']>
    'LazyUiChartsCompactChart': LazyComponent<typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/server-placeholder")['default']>
    'LazyNuxtRouteAnnouncer': LazyComponent<typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/server-placeholder")['default']>
}

declare module 'vue' {
  export interface GlobalComponents extends _GlobalComponents { }
}

export const BrandLogoAnimated: typeof import("../src/components/brand/LogoAnimated.vue")['default']
export const BrandTextLogo: typeof import("../src/components/brand/TextLogo.vue")['default']
export const UiAccordion: typeof import("../src/components/ui/Accordion.vue")['default']
export const UiAdPlaceholder: typeof import("../src/components/ui/AdPlaceholder.vue")['default']
export const UiAutomaticAccordion: typeof import("../src/components/ui/AutomaticAccordion.vue")['default']
export const UiBreadcrumbs: typeof import("../src/components/ui/Breadcrumbs.vue")['default']
export const UiChips: typeof import("../src/components/ui/Chips.vue")['default']
export const UiConditionalNuxtLink: typeof import("../src/components/ui/ConditionalNuxtLink.vue")['default']
export const UiDoubleIcon: typeof import("../src/components/ui/DoubleIcon.vue")['default']
export const UiEnvironmentIndicator: typeof import("../src/components/ui/EnvironmentIndicator.vue")['default']
export const UiFileInput: typeof import("../src/components/ui/FileInput.vue")['default']
export const UiHCaptcha: typeof import("../src/components/ui/HCaptcha.vue")['default']
export const UiMessageBanner: typeof import("../src/components/ui/MessageBanner.vue")['default']
export const UiModal: typeof import("../src/components/ui/Modal.vue")['default']
export const UiNavRow: typeof import("../src/components/ui/NavRow.vue")['default']
export const UiNavStack: typeof import("../src/components/ui/NavStack.vue")['default']
export const UiNavTabs: typeof import("../src/components/ui/NavTabs.vue")['default']
export const UiNewsletterButton: typeof import("../src/components/ui/NewsletterButton.vue")['default']
export const UiNotificationItem: typeof import("../src/components/ui/NotificationItem.vue")['default']
export const UiOptionGroup: typeof import("../src/components/ui/OptionGroup.vue")['default']
export const UiOrganizationProjectTransferModal: typeof import("../src/components/ui/OrganizationProjectTransferModal.vue")['default']
export const UiProjectCard: typeof import("../src/components/ui/ProjectCard.vue")['default']
export const UiProjectMemberHeader: typeof import("../src/components/ui/ProjectMemberHeader.vue")['default']
export const UiShareArticleButtons: typeof import("../src/components/ui/ShareArticleButtons.vue")['default']
export const UiAdminBatchCreditModal: typeof import("../src/components/ui/admin/BatchCreditModal.vue")['default']
export const UiChartsChart: typeof import("../src/components/ui/charts/Chart.client.vue")['default']
export const UiChartsChartDisplay: typeof import("../src/components/ui/charts/ChartDisplay.vue")['default']
export const UiChartsCompactChart: typeof import("../src/components/ui/charts/CompactChart.client.vue")['default']
export const UiCreateCollectionCreateModal: typeof import("../src/components/ui/create/CollectionCreateModal.vue")['default']
export const UiCreateLimitAlert: typeof import("../src/components/ui/create/CreateLimitAlert.vue")['default']
export const UiCreateOrganizationCreateModal: typeof import("../src/components/ui/create/OrganizationCreateModal.vue")['default']
export const UiCreateProjectCreateModal: typeof import("../src/components/ui/create/ProjectCreateModal.vue")['default']
export const UiDashboardCreatorTaxFormModal: typeof import("../src/components/ui/dashboard/CreatorTaxFormModal.vue")['default']
export const UiDashboardCreatorWithdrawModal: typeof import("../src/components/ui/dashboard/CreatorWithdrawModal.vue")['default']
export const UiDashboardRevenueInputField: typeof import("../src/components/ui/dashboard/RevenueInputField.vue")['default']
export const UiDashboardRevenueTransaction: typeof import("../src/components/ui/dashboard/RevenueTransaction.vue")['default']
export const UiDashboardWithdrawFeeBreakdown: typeof import("../src/components/ui/dashboard/WithdrawFeeBreakdown.vue")['default']
export const UiDashboardWithdrawStagesCompletionStage: typeof import("../src/components/ui/dashboard/withdraw-stages/CompletionStage.vue")['default']
export const UiDashboardWithdrawStagesLegacyPaypalDetailsStage: typeof import("../src/components/ui/dashboard/withdraw-stages/LegacyPaypalDetailsStage.vue")['default']
export const UiDashboardWithdrawStagesMethodSelectionStage: typeof import("../src/components/ui/dashboard/withdraw-stages/MethodSelectionStage.vue")['default']
export const UiDashboardWithdrawStagesMuralpayDetailsStage: typeof import("../src/components/ui/dashboard/withdraw-stages/MuralpayDetailsStage.vue")['default']
export const UiDashboardWithdrawStagesMuralpayKycStage: typeof import("../src/components/ui/dashboard/withdraw-stages/MuralpayKycStage.vue")['default']
export const UiDashboardWithdrawStagesTaxFormStage: typeof import("../src/components/ui/dashboard/withdraw-stages/TaxFormStage.vue")['default']
export const UiDashboardWithdrawStagesTremendousDetailsStage: typeof import("../src/components/ui/dashboard/withdraw-stages/TremendousDetailsStage.vue")['default']
export const UiModerationDelphiReportCard: typeof import("../src/components/ui/moderation/ModerationDelphiReportCard.vue")['default']
export const UiModerationProjectNags: typeof import("../src/components/ui/moderation/ModerationProjectNags.vue")['default']
export const UiModerationQueueCard: typeof import("../src/components/ui/moderation/ModerationQueueCard.vue")['default']
export const UiModerationReportCard: typeof import("../src/components/ui/moderation/ModerationReportCard.vue")['default']
export const UiModerationChecklistKeybindsModal: typeof import("../src/components/ui/moderation/checklist/ChecklistKeybindsModal.vue")['default']
export const UiModerationChecklist: typeof import("../src/components/ui/moderation/checklist/ModerationChecklist.vue")['default']
export const UiModerationChecklistModpackPermissionsFlow: typeof import("../src/components/ui/moderation/checklist/ModpackPermissionsFlow.vue")['default']
export const UiModrinthLoadingIndicator: typeof import("../src/components/ui/modrinth-loading-indicator")['default']
export const UiNewsLatestNewsRow: typeof import("../src/components/ui/news/LatestNewsRow.vue")['default']
export const UiReportInfo: typeof import("../src/components/ui/report/ReportInfo.vue")['default']
export const UiReportView: typeof import("../src/components/ui/report/ReportView.vue")['default']
export const UiReportReportsList: typeof import("../src/components/ui/report/ReportsList.vue")['default']
export const UiSearchCategories: typeof import("../src/components/ui/search/Categories.vue")['default']
export const UiServersBackupCreateModal: typeof import("../src/components/ui/servers/BackupCreateModal.vue")['default']
export const UiServersBackupDeleteModal: typeof import("../src/components/ui/servers/BackupDeleteModal.vue")['default']
export const UiServersBackupItem: typeof import("../src/components/ui/servers/BackupItem.vue")['default']
export const UiServersBackupRenameModal: typeof import("../src/components/ui/servers/BackupRenameModal.vue")['default']
export const UiServersBackupRestoreModal: typeof import("../src/components/ui/servers/BackupRestoreModal.vue")['default']
export const UiServersBackupSettingsModal: typeof import("../src/components/ui/servers/BackupSettingsModal.vue")['default']
export const UiServersContentVersionEditModal: typeof import("../src/components/ui/servers/ContentVersionEditModal.vue")['default']
export const UiServersContentVersionFilter: typeof import("../src/components/ui/servers/ContentVersionFilter.vue")['default']
export const UiServersFileItem: typeof import("../src/components/ui/servers/FileItem.vue")['default']
export const UiServersFileManagerError: typeof import("../src/components/ui/servers/FileManagerError.vue")['default']
export const UiServersFileVirtualList: typeof import("../src/components/ui/servers/FileVirtualList.vue")['default']
export const UiServersFilesBrowseNavbar: typeof import("../src/components/ui/servers/FilesBrowseNavbar.vue")['default']
export const UiServersFilesContextMenu: typeof import("../src/components/ui/servers/FilesContextMenu.vue")['default']
export const UiServersFilesCreateItemModal: typeof import("../src/components/ui/servers/FilesCreateItemModal.vue")['default']
export const UiServersFilesDeleteItemModal: typeof import("../src/components/ui/servers/FilesDeleteItemModal.vue")['default']
export const UiServersFilesEditingNavbar: typeof import("../src/components/ui/servers/FilesEditingNavbar.vue")['default']
export const UiServersFilesImageViewer: typeof import("../src/components/ui/servers/FilesImageViewer.vue")['default']
export const UiServersFilesLabelBar: typeof import("../src/components/ui/servers/FilesLabelBar.vue")['default']
export const UiServersFilesMoveItemModal: typeof import("../src/components/ui/servers/FilesMoveItemModal.vue")['default']
export const UiServersFilesRenameItemModal: typeof import("../src/components/ui/servers/FilesRenameItemModal.vue")['default']
export const UiServersFilesUploadConflictModal: typeof import("../src/components/ui/servers/FilesUploadConflictModal.vue")['default']
export const UiServersFilesUploadDragAndDrop: typeof import("../src/components/ui/servers/FilesUploadDragAndDrop.vue")['default']
export const UiServersFilesUploadDropdown: typeof import("../src/components/ui/servers/FilesUploadDropdown.vue")['default']
export const UiServersFilesUploadZipUrlModal: typeof import("../src/components/ui/servers/FilesUploadZipUrlModal.vue")['default']
export const UiServersGlobe: typeof import("../src/components/ui/servers/Globe.vue")['default']
export const UiServersInstallingTicker: typeof import("../src/components/ui/servers/InstallingTicker.vue")['default']
export const UiServersLoaderSelector: typeof import("../src/components/ui/servers/LoaderSelector.vue")['default']
export const UiServersLoaderSelectorCard: typeof import("../src/components/ui/servers/LoaderSelectorCard.vue")['default']
export const UiServersLogLine: typeof import("../src/components/ui/servers/LogLine.vue")['default']
export const UiServersModrinthServersIcon: typeof import("../src/components/ui/servers/ModrinthServersIcon.vue")['default']
export const UiServersPanelServerActionButton: typeof import("../src/components/ui/servers/PanelServerActionButton.vue")['default']
export const UiServersPanelServerStatus: typeof import("../src/components/ui/servers/PanelServerStatus.vue")['default']
export const UiServersPanelSpinner: typeof import("../src/components/ui/servers/PanelSpinner.vue")['default']
export const UiServersPanelTerminal: typeof import("../src/components/ui/servers/PanelTerminal.vue")['default']
export const UiServersPlatformChangeModpackVersionModal: typeof import("../src/components/ui/servers/PlatformChangeModpackVersionModal.vue")['default']
export const UiServersPlatformMrpackModal: typeof import("../src/components/ui/servers/PlatformMrpackModal.vue")['default']
export const UiServersPlatformVersionSelectModal: typeof import("../src/components/ui/servers/PlatformVersionSelectModal.vue")['default']
export const UiServersSaveBanner: typeof import("../src/components/ui/servers/SaveBanner.vue")['default']
export const UiServersServerInstallation: typeof import("../src/components/ui/servers/ServerInstallation.vue")['default']
export const UiServersServerSidebar: typeof import("../src/components/ui/servers/ServerSidebar.vue")['default']
export const UiServersServerStats: typeof import("../src/components/ui/servers/ServerStats.vue")['default']
export const UiServersUpgradeModalWrapper: typeof import("../src/components/ui/servers/ServersUpgradeModalWrapper.vue")['default']
export const UiServersTeleportOverflowMenu: typeof import("../src/components/ui/servers/TeleportOverflowMenu.vue")['default']
export const UiServersIconsChevronDownIcon: typeof import("../src/components/ui/servers/icons/ChevronDownIcon.vue")['default']
export const UiServersIconsChevronUpIcon: typeof import("../src/components/ui/servers/icons/ChevronUpIcon.vue")['default']
export const UiServersIconsCodeFileIcon: typeof import("../src/components/ui/servers/icons/CodeFileIcon.vue")['default']
export const UiServersIconsCogFolderIcon: typeof import("../src/components/ui/servers/icons/CogFolderIcon.vue")['default']
export const UiServersIconsEarthIcon: typeof import("../src/components/ui/servers/icons/EarthIcon.vue")['default']
export const UiServersIconsFullscreenIcon: typeof import("../src/components/ui/servers/icons/FullscreenIcon.vue")['default']
export const UiServersIconsImageFileIcon: typeof import("../src/components/ui/servers/icons/ImageFileIcon.vue")['default']
export const UiServersIconsLoaderIcon: typeof import("../src/components/ui/servers/icons/LoaderIcon.vue")['default']
export const UiServersIconsLoadingIcon: typeof import("../src/components/ui/servers/icons/LoadingIcon.vue")['default']
export const UiServersIconsMinimizeIconVue: typeof import("../src/components/ui/servers/icons/MinimizeIcon.vue.vue")['default']
export const UiServersIconsPanelErrorIcon: typeof import("../src/components/ui/servers/icons/PanelErrorIcon.vue")['default']
export const UiServersIconsSlashIcon: typeof import("../src/components/ui/servers/icons/SlashIcon.vue")['default']
export const UiServersIconsTextFileIcon: typeof import("../src/components/ui/servers/icons/TextFileIcon.vue")['default']
export const UiServersIconsTimer: typeof import("../src/components/ui/servers/icons/Timer.vue")['default']
export const UiServersMarketingMedalPlanPromotion: typeof import("../src/components/ui/servers/marketing/MedalPlanPromotion.vue")['default']
export const UiServersMarketingMedalServerCountdown: typeof import("../src/components/ui/servers/marketing/MedalServerCountdown.vue")['default']
export const UiServersMarketingServerPlanSelector: typeof import("../src/components/ui/servers/marketing/ServerPlanSelector.vue")['default']
export const UiServersNoticeAssignNoticeModal: typeof import("../src/components/ui/servers/notice/AssignNoticeModal.vue")['default']
export const UiServersNoticeDashboardItem: typeof import("../src/components/ui/servers/notice/NoticeDashboardItem.vue")['default']
export const UiThreadConversationThread: typeof import("../src/components/ui/thread/ConversationThread.vue")['default']
export const UiThreadReportThread: typeof import("../src/components/ui/thread/ReportThread.vue")['default']
export const UiThreadMessage: typeof import("../src/components/ui/thread/ThreadMessage.vue")['default']
export const UiThreadSummary: typeof import("../src/components/ui/thread/ThreadSummary.vue")['default']
export const NuxtWelcome: typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/welcome.vue")['default']
export const NuxtLayout: typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/nuxt-layout")['default']
export const NuxtErrorBoundary: typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/nuxt-error-boundary.vue")['default']
export const ClientOnly: typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/client-only")['default']
export const DevOnly: typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/dev-only")['default']
export const ServerPlaceholder: typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/server-placeholder")['default']
export const NuxtLink: typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/nuxt-link")['default']
export const NuxtLoadingIndicator: typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/nuxt-loading-indicator")['default']
export const NuxtTime: typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/nuxt-time.vue")['default']
export const NuxtRouteAnnouncer: typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/nuxt-route-announcer")['default']
export const NuxtImg: typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/nuxt-stubs")['NuxtImg']
export const NuxtPicture: typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/nuxt-stubs")['NuxtPicture']
export const IntlFormatted: typeof import("@vintl/vintl/components")['IntlFormatted']
export const NuxtPage: typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/pages/runtime/page")['default']
export const NoScript: typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/head/runtime/components")['NoScript']
export const Link: typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/head/runtime/components")['Link']
export const Base: typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/head/runtime/components")['Base']
export const Title: typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/head/runtime/components")['Title']
export const Meta: typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/head/runtime/components")['Meta']
export const Style: typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/head/runtime/components")['Style']
export const Head: typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/head/runtime/components")['Head']
export const Html: typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/head/runtime/components")['Html']
export const Body: typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/head/runtime/components")['Body']
export const NuxtIsland: typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/nuxt-island")['default']
export const UiChartsChart: typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/server-placeholder")['default']
export const UiChartsCompactChart: typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/server-placeholder")['default']
export const NuxtRouteAnnouncer: typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/server-placeholder")['default']
export const LazyBrandLogoAnimated: LazyComponent<typeof import("../src/components/brand/LogoAnimated.vue")['default']>
export const LazyBrandTextLogo: LazyComponent<typeof import("../src/components/brand/TextLogo.vue")['default']>
export const LazyUiAccordion: LazyComponent<typeof import("../src/components/ui/Accordion.vue")['default']>
export const LazyUiAdPlaceholder: LazyComponent<typeof import("../src/components/ui/AdPlaceholder.vue")['default']>
export const LazyUiAutomaticAccordion: LazyComponent<typeof import("../src/components/ui/AutomaticAccordion.vue")['default']>
export const LazyUiBreadcrumbs: LazyComponent<typeof import("../src/components/ui/Breadcrumbs.vue")['default']>
export const LazyUiChips: LazyComponent<typeof import("../src/components/ui/Chips.vue")['default']>
export const LazyUiConditionalNuxtLink: LazyComponent<typeof import("../src/components/ui/ConditionalNuxtLink.vue")['default']>
export const LazyUiDoubleIcon: LazyComponent<typeof import("../src/components/ui/DoubleIcon.vue")['default']>
export const LazyUiEnvironmentIndicator: LazyComponent<typeof import("../src/components/ui/EnvironmentIndicator.vue")['default']>
export const LazyUiFileInput: LazyComponent<typeof import("../src/components/ui/FileInput.vue")['default']>
export const LazyUiHCaptcha: LazyComponent<typeof import("../src/components/ui/HCaptcha.vue")['default']>
export const LazyUiMessageBanner: LazyComponent<typeof import("../src/components/ui/MessageBanner.vue")['default']>
export const LazyUiModal: LazyComponent<typeof import("../src/components/ui/Modal.vue")['default']>
export const LazyUiNavRow: LazyComponent<typeof import("../src/components/ui/NavRow.vue")['default']>
export const LazyUiNavStack: LazyComponent<typeof import("../src/components/ui/NavStack.vue")['default']>
export const LazyUiNavTabs: LazyComponent<typeof import("../src/components/ui/NavTabs.vue")['default']>
export const LazyUiNewsletterButton: LazyComponent<typeof import("../src/components/ui/NewsletterButton.vue")['default']>
export const LazyUiNotificationItem: LazyComponent<typeof import("../src/components/ui/NotificationItem.vue")['default']>
export const LazyUiOptionGroup: LazyComponent<typeof import("../src/components/ui/OptionGroup.vue")['default']>
export const LazyUiOrganizationProjectTransferModal: LazyComponent<typeof import("../src/components/ui/OrganizationProjectTransferModal.vue")['default']>
export const LazyUiProjectCard: LazyComponent<typeof import("../src/components/ui/ProjectCard.vue")['default']>
export const LazyUiProjectMemberHeader: LazyComponent<typeof import("../src/components/ui/ProjectMemberHeader.vue")['default']>
export const LazyUiShareArticleButtons: LazyComponent<typeof import("../src/components/ui/ShareArticleButtons.vue")['default']>
export const LazyUiAdminBatchCreditModal: LazyComponent<typeof import("../src/components/ui/admin/BatchCreditModal.vue")['default']>
export const LazyUiChartsChart: LazyComponent<typeof import("../src/components/ui/charts/Chart.client.vue")['default']>
export const LazyUiChartsChartDisplay: LazyComponent<typeof import("../src/components/ui/charts/ChartDisplay.vue")['default']>
export const LazyUiChartsCompactChart: LazyComponent<typeof import("../src/components/ui/charts/CompactChart.client.vue")['default']>
export const LazyUiCreateCollectionCreateModal: LazyComponent<typeof import("../src/components/ui/create/CollectionCreateModal.vue")['default']>
export const LazyUiCreateLimitAlert: LazyComponent<typeof import("../src/components/ui/create/CreateLimitAlert.vue")['default']>
export const LazyUiCreateOrganizationCreateModal: LazyComponent<typeof import("../src/components/ui/create/OrganizationCreateModal.vue")['default']>
export const LazyUiCreateProjectCreateModal: LazyComponent<typeof import("../src/components/ui/create/ProjectCreateModal.vue")['default']>
export const LazyUiDashboardCreatorTaxFormModal: LazyComponent<typeof import("../src/components/ui/dashboard/CreatorTaxFormModal.vue")['default']>
export const LazyUiDashboardCreatorWithdrawModal: LazyComponent<typeof import("../src/components/ui/dashboard/CreatorWithdrawModal.vue")['default']>
export const LazyUiDashboardRevenueInputField: LazyComponent<typeof import("../src/components/ui/dashboard/RevenueInputField.vue")['default']>
export const LazyUiDashboardRevenueTransaction: LazyComponent<typeof import("../src/components/ui/dashboard/RevenueTransaction.vue")['default']>
export const LazyUiDashboardWithdrawFeeBreakdown: LazyComponent<typeof import("../src/components/ui/dashboard/WithdrawFeeBreakdown.vue")['default']>
export const LazyUiDashboardWithdrawStagesCompletionStage: LazyComponent<typeof import("../src/components/ui/dashboard/withdraw-stages/CompletionStage.vue")['default']>
export const LazyUiDashboardWithdrawStagesLegacyPaypalDetailsStage: LazyComponent<typeof import("../src/components/ui/dashboard/withdraw-stages/LegacyPaypalDetailsStage.vue")['default']>
export const LazyUiDashboardWithdrawStagesMethodSelectionStage: LazyComponent<typeof import("../src/components/ui/dashboard/withdraw-stages/MethodSelectionStage.vue")['default']>
export const LazyUiDashboardWithdrawStagesMuralpayDetailsStage: LazyComponent<typeof import("../src/components/ui/dashboard/withdraw-stages/MuralpayDetailsStage.vue")['default']>
export const LazyUiDashboardWithdrawStagesMuralpayKycStage: LazyComponent<typeof import("../src/components/ui/dashboard/withdraw-stages/MuralpayKycStage.vue")['default']>
export const LazyUiDashboardWithdrawStagesTaxFormStage: LazyComponent<typeof import("../src/components/ui/dashboard/withdraw-stages/TaxFormStage.vue")['default']>
export const LazyUiDashboardWithdrawStagesTremendousDetailsStage: LazyComponent<typeof import("../src/components/ui/dashboard/withdraw-stages/TremendousDetailsStage.vue")['default']>
export const LazyUiModerationDelphiReportCard: LazyComponent<typeof import("../src/components/ui/moderation/ModerationDelphiReportCard.vue")['default']>
export const LazyUiModerationProjectNags: LazyComponent<typeof import("../src/components/ui/moderation/ModerationProjectNags.vue")['default']>
export const LazyUiModerationQueueCard: LazyComponent<typeof import("../src/components/ui/moderation/ModerationQueueCard.vue")['default']>
export const LazyUiModerationReportCard: LazyComponent<typeof import("../src/components/ui/moderation/ModerationReportCard.vue")['default']>
export const LazyUiModerationChecklistKeybindsModal: LazyComponent<typeof import("../src/components/ui/moderation/checklist/ChecklistKeybindsModal.vue")['default']>
export const LazyUiModerationChecklist: LazyComponent<typeof import("../src/components/ui/moderation/checklist/ModerationChecklist.vue")['default']>
export const LazyUiModerationChecklistModpackPermissionsFlow: LazyComponent<typeof import("../src/components/ui/moderation/checklist/ModpackPermissionsFlow.vue")['default']>
export const LazyUiModrinthLoadingIndicator: LazyComponent<typeof import("../src/components/ui/modrinth-loading-indicator")['default']>
export const LazyUiNewsLatestNewsRow: LazyComponent<typeof import("../src/components/ui/news/LatestNewsRow.vue")['default']>
export const LazyUiReportInfo: LazyComponent<typeof import("../src/components/ui/report/ReportInfo.vue")['default']>
export const LazyUiReportView: LazyComponent<typeof import("../src/components/ui/report/ReportView.vue")['default']>
export const LazyUiReportReportsList: LazyComponent<typeof import("../src/components/ui/report/ReportsList.vue")['default']>
export const LazyUiSearchCategories: LazyComponent<typeof import("../src/components/ui/search/Categories.vue")['default']>
export const LazyUiServersBackupCreateModal: LazyComponent<typeof import("../src/components/ui/servers/BackupCreateModal.vue")['default']>
export const LazyUiServersBackupDeleteModal: LazyComponent<typeof import("../src/components/ui/servers/BackupDeleteModal.vue")['default']>
export const LazyUiServersBackupItem: LazyComponent<typeof import("../src/components/ui/servers/BackupItem.vue")['default']>
export const LazyUiServersBackupRenameModal: LazyComponent<typeof import("../src/components/ui/servers/BackupRenameModal.vue")['default']>
export const LazyUiServersBackupRestoreModal: LazyComponent<typeof import("../src/components/ui/servers/BackupRestoreModal.vue")['default']>
export const LazyUiServersBackupSettingsModal: LazyComponent<typeof import("../src/components/ui/servers/BackupSettingsModal.vue")['default']>
export const LazyUiServersContentVersionEditModal: LazyComponent<typeof import("../src/components/ui/servers/ContentVersionEditModal.vue")['default']>
export const LazyUiServersContentVersionFilter: LazyComponent<typeof import("../src/components/ui/servers/ContentVersionFilter.vue")['default']>
export const LazyUiServersFileItem: LazyComponent<typeof import("../src/components/ui/servers/FileItem.vue")['default']>
export const LazyUiServersFileManagerError: LazyComponent<typeof import("../src/components/ui/servers/FileManagerError.vue")['default']>
export const LazyUiServersFileVirtualList: LazyComponent<typeof import("../src/components/ui/servers/FileVirtualList.vue")['default']>
export const LazyUiServersFilesBrowseNavbar: LazyComponent<typeof import("../src/components/ui/servers/FilesBrowseNavbar.vue")['default']>
export const LazyUiServersFilesContextMenu: LazyComponent<typeof import("../src/components/ui/servers/FilesContextMenu.vue")['default']>
export const LazyUiServersFilesCreateItemModal: LazyComponent<typeof import("../src/components/ui/servers/FilesCreateItemModal.vue")['default']>
export const LazyUiServersFilesDeleteItemModal: LazyComponent<typeof import("../src/components/ui/servers/FilesDeleteItemModal.vue")['default']>
export const LazyUiServersFilesEditingNavbar: LazyComponent<typeof import("../src/components/ui/servers/FilesEditingNavbar.vue")['default']>
export const LazyUiServersFilesImageViewer: LazyComponent<typeof import("../src/components/ui/servers/FilesImageViewer.vue")['default']>
export const LazyUiServersFilesLabelBar: LazyComponent<typeof import("../src/components/ui/servers/FilesLabelBar.vue")['default']>
export const LazyUiServersFilesMoveItemModal: LazyComponent<typeof import("../src/components/ui/servers/FilesMoveItemModal.vue")['default']>
export const LazyUiServersFilesRenameItemModal: LazyComponent<typeof import("../src/components/ui/servers/FilesRenameItemModal.vue")['default']>
export const LazyUiServersFilesUploadConflictModal: LazyComponent<typeof import("../src/components/ui/servers/FilesUploadConflictModal.vue")['default']>
export const LazyUiServersFilesUploadDragAndDrop: LazyComponent<typeof import("../src/components/ui/servers/FilesUploadDragAndDrop.vue")['default']>
export const LazyUiServersFilesUploadDropdown: LazyComponent<typeof import("../src/components/ui/servers/FilesUploadDropdown.vue")['default']>
export const LazyUiServersFilesUploadZipUrlModal: LazyComponent<typeof import("../src/components/ui/servers/FilesUploadZipUrlModal.vue")['default']>
export const LazyUiServersGlobe: LazyComponent<typeof import("../src/components/ui/servers/Globe.vue")['default']>
export const LazyUiServersInstallingTicker: LazyComponent<typeof import("../src/components/ui/servers/InstallingTicker.vue")['default']>
export const LazyUiServersLoaderSelector: LazyComponent<typeof import("../src/components/ui/servers/LoaderSelector.vue")['default']>
export const LazyUiServersLoaderSelectorCard: LazyComponent<typeof import("../src/components/ui/servers/LoaderSelectorCard.vue")['default']>
export const LazyUiServersLogLine: LazyComponent<typeof import("../src/components/ui/servers/LogLine.vue")['default']>
export const LazyUiServersModrinthServersIcon: LazyComponent<typeof import("../src/components/ui/servers/ModrinthServersIcon.vue")['default']>
export const LazyUiServersPanelServerActionButton: LazyComponent<typeof import("../src/components/ui/servers/PanelServerActionButton.vue")['default']>
export const LazyUiServersPanelServerStatus: LazyComponent<typeof import("../src/components/ui/servers/PanelServerStatus.vue")['default']>
export const LazyUiServersPanelSpinner: LazyComponent<typeof import("../src/components/ui/servers/PanelSpinner.vue")['default']>
export const LazyUiServersPanelTerminal: LazyComponent<typeof import("../src/components/ui/servers/PanelTerminal.vue")['default']>
export const LazyUiServersPlatformChangeModpackVersionModal: LazyComponent<typeof import("../src/components/ui/servers/PlatformChangeModpackVersionModal.vue")['default']>
export const LazyUiServersPlatformMrpackModal: LazyComponent<typeof import("../src/components/ui/servers/PlatformMrpackModal.vue")['default']>
export const LazyUiServersPlatformVersionSelectModal: LazyComponent<typeof import("../src/components/ui/servers/PlatformVersionSelectModal.vue")['default']>
export const LazyUiServersSaveBanner: LazyComponent<typeof import("../src/components/ui/servers/SaveBanner.vue")['default']>
export const LazyUiServersServerInstallation: LazyComponent<typeof import("../src/components/ui/servers/ServerInstallation.vue")['default']>
export const LazyUiServersServerSidebar: LazyComponent<typeof import("../src/components/ui/servers/ServerSidebar.vue")['default']>
export const LazyUiServersServerStats: LazyComponent<typeof import("../src/components/ui/servers/ServerStats.vue")['default']>
export const LazyUiServersUpgradeModalWrapper: LazyComponent<typeof import("../src/components/ui/servers/ServersUpgradeModalWrapper.vue")['default']>
export const LazyUiServersTeleportOverflowMenu: LazyComponent<typeof import("../src/components/ui/servers/TeleportOverflowMenu.vue")['default']>
export const LazyUiServersIconsChevronDownIcon: LazyComponent<typeof import("../src/components/ui/servers/icons/ChevronDownIcon.vue")['default']>
export const LazyUiServersIconsChevronUpIcon: LazyComponent<typeof import("../src/components/ui/servers/icons/ChevronUpIcon.vue")['default']>
export const LazyUiServersIconsCodeFileIcon: LazyComponent<typeof import("../src/components/ui/servers/icons/CodeFileIcon.vue")['default']>
export const LazyUiServersIconsCogFolderIcon: LazyComponent<typeof import("../src/components/ui/servers/icons/CogFolderIcon.vue")['default']>
export const LazyUiServersIconsEarthIcon: LazyComponent<typeof import("../src/components/ui/servers/icons/EarthIcon.vue")['default']>
export const LazyUiServersIconsFullscreenIcon: LazyComponent<typeof import("../src/components/ui/servers/icons/FullscreenIcon.vue")['default']>
export const LazyUiServersIconsImageFileIcon: LazyComponent<typeof import("../src/components/ui/servers/icons/ImageFileIcon.vue")['default']>
export const LazyUiServersIconsLoaderIcon: LazyComponent<typeof import("../src/components/ui/servers/icons/LoaderIcon.vue")['default']>
export const LazyUiServersIconsLoadingIcon: LazyComponent<typeof import("../src/components/ui/servers/icons/LoadingIcon.vue")['default']>
export const LazyUiServersIconsMinimizeIconVue: LazyComponent<typeof import("../src/components/ui/servers/icons/MinimizeIcon.vue.vue")['default']>
export const LazyUiServersIconsPanelErrorIcon: LazyComponent<typeof import("../src/components/ui/servers/icons/PanelErrorIcon.vue")['default']>
export const LazyUiServersIconsSlashIcon: LazyComponent<typeof import("../src/components/ui/servers/icons/SlashIcon.vue")['default']>
export const LazyUiServersIconsTextFileIcon: LazyComponent<typeof import("../src/components/ui/servers/icons/TextFileIcon.vue")['default']>
export const LazyUiServersIconsTimer: LazyComponent<typeof import("../src/components/ui/servers/icons/Timer.vue")['default']>
export const LazyUiServersMarketingMedalPlanPromotion: LazyComponent<typeof import("../src/components/ui/servers/marketing/MedalPlanPromotion.vue")['default']>
export const LazyUiServersMarketingMedalServerCountdown: LazyComponent<typeof import("../src/components/ui/servers/marketing/MedalServerCountdown.vue")['default']>
export const LazyUiServersMarketingServerPlanSelector: LazyComponent<typeof import("../src/components/ui/servers/marketing/ServerPlanSelector.vue")['default']>
export const LazyUiServersNoticeAssignNoticeModal: LazyComponent<typeof import("../src/components/ui/servers/notice/AssignNoticeModal.vue")['default']>
export const LazyUiServersNoticeDashboardItem: LazyComponent<typeof import("../src/components/ui/servers/notice/NoticeDashboardItem.vue")['default']>
export const LazyUiThreadConversationThread: LazyComponent<typeof import("../src/components/ui/thread/ConversationThread.vue")['default']>
export const LazyUiThreadReportThread: LazyComponent<typeof import("../src/components/ui/thread/ReportThread.vue")['default']>
export const LazyUiThreadMessage: LazyComponent<typeof import("../src/components/ui/thread/ThreadMessage.vue")['default']>
export const LazyUiThreadSummary: LazyComponent<typeof import("../src/components/ui/thread/ThreadSummary.vue")['default']>
export const LazyNuxtWelcome: LazyComponent<typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/welcome.vue")['default']>
export const LazyNuxtLayout: LazyComponent<typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/nuxt-layout")['default']>
export const LazyNuxtErrorBoundary: LazyComponent<typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/nuxt-error-boundary.vue")['default']>
export const LazyClientOnly: LazyComponent<typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/client-only")['default']>
export const LazyDevOnly: LazyComponent<typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/dev-only")['default']>
export const LazyServerPlaceholder: LazyComponent<typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/server-placeholder")['default']>
export const LazyNuxtLink: LazyComponent<typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/nuxt-link")['default']>
export const LazyNuxtLoadingIndicator: LazyComponent<typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/nuxt-loading-indicator")['default']>
export const LazyNuxtTime: LazyComponent<typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/nuxt-time.vue")['default']>
export const LazyNuxtRouteAnnouncer: LazyComponent<typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/nuxt-route-announcer")['default']>
export const LazyNuxtImg: LazyComponent<typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/nuxt-stubs")['NuxtImg']>
export const LazyNuxtPicture: LazyComponent<typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/nuxt-stubs")['NuxtPicture']>
export const LazyIntlFormatted: LazyComponent<typeof import("@vintl/vintl/components")['IntlFormatted']>
export const LazyNuxtPage: LazyComponent<typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/pages/runtime/page")['default']>
export const LazyNoScript: LazyComponent<typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/head/runtime/components")['NoScript']>
export const LazyLink: LazyComponent<typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/head/runtime/components")['Link']>
export const LazyBase: LazyComponent<typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/head/runtime/components")['Base']>
export const LazyTitle: LazyComponent<typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/head/runtime/components")['Title']>
export const LazyMeta: LazyComponent<typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/head/runtime/components")['Meta']>
export const LazyStyle: LazyComponent<typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/head/runtime/components")['Style']>
export const LazyHead: LazyComponent<typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/head/runtime/components")['Head']>
export const LazyHtml: LazyComponent<typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/head/runtime/components")['Html']>
export const LazyBody: LazyComponent<typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/head/runtime/components")['Body']>
export const LazyNuxtIsland: LazyComponent<typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/nuxt-island")['default']>
export const LazyUiChartsChart: LazyComponent<typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/server-placeholder")['default']>
export const LazyUiChartsCompactChart: LazyComponent<typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/server-placeholder")['default']>
export const LazyNuxtRouteAnnouncer: LazyComponent<typeof import("../../../node_modules/.pnpm/nuxt@3.18.1_@netlify+blobs@9.1.2_@parcel+watcher@2.5.1_@types+node@20.19.9_@vue+compiler-sfc@_3iiw5cqlngib35pp3gvxzsvasa/node_modules/nuxt/dist/app/components/server-placeholder")['default']>

export const componentNames: string[]
