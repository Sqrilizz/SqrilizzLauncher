import { createRouter, createWebHistory } from 'vue-router'

/**
 * Configures application routing. Add page to pages/index and then add to route table here.
 */
export default new createRouter({
	history: createWebHistory(),
	routes: [
		{
			path: '/',
			name: 'Home',
			component: () => import('@/pages/Index.vue'),
			meta: {
				breadcrumb: [{ name: 'Home' }],
			},
		},
		{
			path: '/worlds',
			name: 'Worlds',
			component: () => import('@/pages/Worlds.vue'),
			meta: {
				breadcrumb: [{ name: 'Worlds' }],
			},
		},
		{
			path: '/servers/manage/',
			name: 'Servers',
			component: () =>
				import('@modrinth/ui/pages').then(({ ServersManagePageIndex }) => ServersManagePageIndex),
			meta: {
				breadcrumb: [{ name: 'Servers' }],
			},
		},
		{
			path: '/browse/:projectType',
			name: 'Discover content',
			component: () => import('@/pages/Browse.vue'),
			meta: {
				breadcrumb: [{ name: 'Discover content' }],
			},
		},
		{
			path: '/skins',
			name: 'Skins',
			component: () => import('@/pages/Skins.vue'),
			meta: {
				breadcrumb: [{ name: 'Skins' }],
			},
		},
		{
			path: '/library',
			name: 'Library',
			component: () => import('@/pages/library/Index.vue'),
			meta: {
				breadcrumb: [{ name: 'Library' }],
			},
			children: [
				{
					path: '',
					name: 'Overview',
					component: () => import('@/pages/library/Overview.vue'),
				},
				{
					path: 'downloaded',
					name: 'Downloaded',
					component: () => import('@/pages/library/Downloaded.vue'),
				},
				{
					path: 'custom',
					name: 'Custom',
					component: () => import('@/pages/library/Custom.vue'),
				},
			],
		},
		{
			path: '/project/:id',
			name: 'Project',
			component: () => import('@/pages/project/Index.vue'),
			props: true,
			children: [
				{
					path: '',
					name: 'Description',
					component: () => import('@/pages/project/Description.vue'),
					meta: {
						useContext: true,
						breadcrumb: [{ name: '?Project' }],
					},
				},
				{
					path: 'versions',
					name: 'Versions',
					component: () => import('@/pages/project/Versions.vue'),
					meta: {
						useContext: true,
						breadcrumb: [{ name: '?Project', link: '/project/{id}/' }, { name: 'Versions' }],
					},
				},
				{
					path: 'version/:version',
					name: 'Version',
					component: () => import('@/pages/project/Version.vue'),
					props: true,
					meta: {
						useContext: true,
						breadcrumb: [
							{ name: '?Project', link: '/project/{id}/' },
							{ name: 'Versions', link: '/project/{id}/versions' },
							{ name: '?Version' },
						],
					},
				},
				{
					path: 'gallery',
					name: 'Gallery',
					component: () => import('@/pages/project/Gallery.vue'),
					meta: {
						useContext: true,
						breadcrumb: [{ name: '?Project', link: '/project/{id}/' }, { name: 'Gallery' }],
					},
				},
			],
		},
		{
			path: '/instance/:id',
			name: 'Instance',
			component: () => import('@/pages/instance/Index.vue'),
			props: true,
			children: [
				// {
				//   path: '',
				//   name: 'Overview',
				//   component: Instance.Overview,
				//   meta: {
				//     useRootContext: true,
				//     breadcrumb: [{ name: '?Instance' }],
				//   },
				// },
				{
					path: 'worlds',
					name: 'InstanceWorlds',
					component: () => import('@/pages/instance/Worlds.vue'),
					meta: {
						useRootContext: true,
						breadcrumb: [{ name: '?Instance', link: '/instance/{id}/' }, { name: 'Worlds' }],
					},
				},
				{
					path: '',
					name: 'Mods',
					component: () => import('@/pages/instance/Mods.vue'),
					meta: {
						useRootContext: true,
						breadcrumb: [{ name: '?Instance', link: '/instance/{id}/' }, { name: 'Mods' }],
					},
				},
				{
					path: 'projects/:type',
					name: 'ModsFilter',
					component: () => import('@/pages/instance/Mods.vue'),
					meta: {
						useRootContext: true,
						breadcrumb: [{ name: '?Instance', link: '/instance/{id}/' }, { name: 'Mods' }],
					},
				},
				{
					path: 'logs',
					name: 'Logs',
					component: () => import('@/pages/instance/Logs.vue'),
					meta: {
						useRootContext: true,
						breadcrumb: [{ name: '?Instance', link: '/instance/{id}/' }, { name: 'Logs' }],
					},
				},
			],
		},
	],
	linkActiveClass: 'router-link-active',
	linkExactActiveClass: 'router-link-exact-active',
	scrollBehavior() {
		// Sometimes Vue's scroll behavior is not working as expected, so we need to manually scroll to top (especially on Linux)
		document.querySelector('.app-viewport')?.scrollTo(0, 0)
		return {
			el: '.app-viewport',
			top: 0,
		}
	},
})
