<!-- src/routes/+layout.svelte -->
<script lang="ts">
	import "../app.css";
	import "$lib/styles/scroll.css";
	import SplashScreen from '$lib/components/SplashScreen.svelte';
	import { onMount } from 'svelte';
	import { invoke } from "@tauri-apps/api/core";
	import { browser } from '$app/environment';
	import { loadFormatsData } from '$lib/data/formats';
	import { getCurrentWebview } from '@tauri-apps/api/webview';
	import { goto } from '$app/navigation';
	import { setLocale } from '$lib/paraglide/runtime';
	import { page } from '$app/state';
	import { applyTheme, watchSystemTheme } from '$lib/data/settings';
	import { create_popup } from '$lib/utils/context-menu';

	let { children } = $props();
	const isHome = browser && window.location.pathname === '/';
	const appReady = browser && sessionStorage.getItem('app-ready') === 'true';
	let showSplash = $state(isHome && !appReady);
	let splashDone = $state(appReady);

	let lang = $derived(page.data?.settings?.language ?? 'en');
	let settings = $derived(page.data?.settings);

	// Переменная для очистки обработчика
	let cleanupContextMenu: (() => void) | null = null;

	$effect(() => {
		if (settings?.theme) {
			applyTheme(settings.theme);
		}
	});

	$effect(() => {
		if (lang) {
			setLocale(lang, { reload: false });
		}
	});

	onMount(async () => {
		loadFormatsData();

		getCurrentWebview().listen('navigate', (event) => {
			goto(event.payload as string);
		});

		if (!appReady) {
			await invoke('app_ready').catch(console.error);

			if (!isHome) {
				onSplashComplete();
			}
		}

		// Удаляем старый обработчик если есть
		if (cleanupContextMenu) {
			cleanupContextMenu();
			cleanupContextMenu = null;
		}

		// 🔥 СОЗДАЕМ МЕНЮ НА КЛИЕНТЕ
		const contextMenuHandler = async (e: MouseEvent) => {
			e.preventDefault();
			e.stopPropagation();
			
			const target = e.target as HTMLElement;
			
			// Проверяем специальные атрибуты
			const isIgnored = target.closest?.('[data-context-menu="ignore"]');
			const isFileItem = target.closest?.('[data-file-item]');
			const hasCustomHandler = target.closest?.('[data-context-menu-handler]');
			
			if (hasCustomHandler) return;
			if (isFileItem) return;
			if (isIgnored) return;

			await create_popup();
		};

		window.addEventListener('contextmenu', contextMenuHandler, true);

		// Сохраняем функцию для очистки
		cleanupContextMenu = () => {
			window.removeEventListener('contextmenu', contextMenuHandler, true);
		};
	});

	onMount(() => {
		if (browser && settings?.theme === 'system' ) {
			const unwatch = watchSystemTheme(() => {
				if (settings?.theme === 'system') {
					applyTheme('system');
				}
			});

			return unwatch;
		}
	});

	async function onSplashComplete() {
		splashDone = true;
		showSplash = false;

		if (browser) {
			sessionStorage.setItem('app-ready', 'true');
		}

		if (settings?.theme) {
			applyTheme(settings.theme);
		}
	}
</script>

{#if showSplash && !splashDone}
	<SplashScreen onComplete={onSplashComplete} />
{/if}

{#key lang}
	<div class="bg-background text-foreground">{@render children?.()}</div>
{/key}