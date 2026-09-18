# ARCHITECTURE.md — Архитектурный разбор модуля app (Android)

## 1. Дерево файлов модуля app

`	ext
app/
    .gitignore
    build.gradle.kts
    lint-baseline.xml
    proguard-rules.pro
    src/
        main/
            AndroidManifest.xml
            aidl/
                io/
                    nekohasekai/
                        sagernet/
                            aidl/
                                ISagerNetService.aidl
                                ISagerNetServiceCallback.aidl
                                SpeedDisplayData.aidl
                                SpeedTestData.aidl
                                TrafficData.aidl
                                TrafficDataBatch.aidl
            assets/
                LICENSE
                list_catalog.json
                metacubexd.tgz
                metacubexd.version.txt
                flags/
                    LICENSE
                    1x1/
                        ad.svg
                        ae.svg
                        af.svg
                        ag.svg
                        ai.svg
                        al.svg
                        am.svg
                        ao.svg
                        aq.svg
                        ar.svg
                        arab.svg
                        as.svg
                        asean.svg
                        at.svg
                        au.svg
                        aw.svg
                        ax.svg
                        az.svg
                        ba.svg
                        bb.svg
                        bd.svg
                        be.svg
                        bf.svg
                        bg.svg
                        bh.svg
                        bi.svg
                        bj.svg
                        bl.svg
                        bm.svg
                        bn.svg
                        bo.svg
                        bq.svg
                        br.svg
                        bs.svg
                        bt.svg
                        bv.svg
                        bw.svg
                        by.svg
                        bz.svg
                        ca.svg
                        cc.svg
                        cd.svg
                        cefta.svg
                        cf.svg
                        cg.svg
                        ch.svg
                        ci.svg
                        ck.svg
                        cl.svg
                        cm.svg
                        cn.svg
                        co.svg
                        cp.svg
                        cr.svg
                        cu.svg
                        cv.svg
                        cw.svg
                        cx.svg
                        cy.svg
                        cz.svg
                        de.svg
                        dg.svg
                        dj.svg
                        dk.svg
                        dm.svg
                        do.svg
                        dz.svg
                        eac.svg
                        ec.svg
                        ee.svg
                        eg.svg
                        eh.svg
                        er.svg
                        es-ct.svg
                        es-ga.svg
                        es-pv.svg
                        es.svg
                        et.svg
                        eu.svg
                        fi.svg
                        fj.svg
                        fk.svg
                        fm.svg
                        fo.svg
                        fr.svg
                        ga.svg
                        gb-eng.svg
                        gb-nir.svg
                        gb-sct.svg
                        gb-wls.svg
                        gb.svg
                        gd.svg
                        ge.svg
                        gf.svg
                        gg.svg
                        gh.svg
                        gi.svg
                        gl.svg
                        gm.svg
                        gn.svg
                        gp.svg
                        gq.svg
                        gr.svg
                        gs.svg
                        gt.svg
                        gu.svg
                        gw.svg
                        gy.svg
                        hk.svg
                        hm.svg
                        hn.svg
                        hr.svg
                        ht.svg
                        hu.svg
                        ic.svg
                        id.svg
                        ie.svg
                        il.svg
                        im.svg
                        in.svg
                        io.svg
                        iq.svg
                        ir.svg
                        is.svg
                        it.svg
                        je.svg
                        jm.svg
                        jo.svg
                        jp.svg
                        ke.svg
                        kg.svg
                        kh.svg
                        ki.svg
                        km.svg
                        kn.svg
                        kp.svg
                        kr.svg
                        kw.svg
                        ky.svg
                        kz.svg
                        la.svg
                        lb.svg
                        lc.svg
                        li.svg
                        lk.svg
                        lr.svg
                        ls.svg
                        lt.svg
                        lu.svg
                        lv.svg
                        ly.svg
                        ma.svg
                        mc.svg
                        md.svg
                        me.svg
                        mf.svg
                        mg.svg
                        mh.svg
                        mk.svg
                        ml.svg
                        mm.svg
                        mn.svg
                        mo.svg
                        mp.svg
                        mq.svg
                        mr.svg
                        ms.svg
                        mt.svg
                        mu.svg
                        mv.svg
                        mw.svg
                        mx.svg
                        my.svg
                        mz.svg
                        na.svg
                        nc.svg
                        ne.svg
                        nf.svg
                        ng.svg
                        ni.svg
                        nl.svg
                        no.svg
                        np.svg
                        nr.svg
                        nu.svg
                        nz.svg
                        om.svg
                        pa.svg
                        pc.svg
                        pe.svg
                        pf.svg
                        pg.svg
                        ph.svg
                        pk.svg
                        pl.svg
                        pm.svg
                        pn.svg
                        pr.svg
                        ps.svg
                        pt.svg
                        pw.svg
                        py.svg
                        qa.svg
                        re.svg
                        ro.svg
                        rs.svg
                        ru.svg
                        rw.svg
                        sa.svg
                        sb.svg
                        sc.svg
                        sd.svg
                        se.svg
                        sg.svg
                        sh-ac.svg
                        sh-hl.svg
                        sh-ta.svg
                        sh.svg
                        si.svg
                        sj.svg
                        sk.svg
                        sl.svg
                        sm.svg
                        sn.svg
                        so.svg
                        sr.svg
                        ss.svg
                        st.svg
                        sv.svg
                        sx.svg
                        sy.svg
                        sz.svg
                        tc.svg
                        td.svg
                        tf.svg
                        tg.svg
                        th.svg
                        tj.svg
                        tk.svg
                        tl.svg
                        tm.svg
                        tn.svg
                        to.svg
                        tr.svg
                        tt.svg
                        tv.svg
                        tw.svg
                        tz.svg
                        ua.svg
                        ug.svg
                        um.svg
                        un.svg
                        us.svg
                        uy.svg
                        uz.svg
                        va.svg
                        vc.svg
                        ve.svg
                        vg.svg
                        vi.svg
                        vn.svg
                        vu.svg
                        wf.svg
                        ws.svg
                        xk.svg
                        xx.svg
                        ye.svg
                        yt.svg
                        za.svg
                        zm.svg
                        zw.svg
                routing/
                    browsers.txt
                    cn/
                        proxy.txt
                    ir/
                        proxy.txt
                    other/
                        proxy.txt
                    ru/
                        direct.txt
                        proxy.txt
            java/
                com/
                    github/
                        shadowsocks/
                            plugin/
                                Utils.kt
                                fragment/
                                    AlertDialogFragment.kt
                io/
                    nekohasekai/
                        sagernet/
                            AndroidTunPayload.kt
                            AppIconManager.kt
                            AppLogLevel.kt
                            AppVersionCachePolicy.kt
                            BatteryOptimization.kt
                            BootReceiver.kt
                            BootReceiverPolicy.kt
                            Constants.kt
                            LocalNetworkPermission.kt
                            LogcatRetentionSize.kt
                            QuickToggleShortcut.kt
                            SagerNet.kt
                            aidl/
                                SpeedDisplayData.kt
                                SpeedTestData.kt
                                TrafficData.kt
                                TrafficDataBatch.kt
                            backup/
                                BackupContainerCodec.kt
                                GitBackupConfigStore.kt
                                GitBackupErrorClassifier.kt
                                GitBackupModels.kt
                                GitBackupRepository.kt
                                GitHttpConnectionFactory.kt
                            bg/
                                AbstractInstance.kt
                                AutomaticConnectionTestPolicy.kt
                                BaseService.kt
                                ConnectionRecoveryPolicy.kt
                                ConnectionTestSessionState.kt
                                CoreOverloadDetector.kt
                                CoreOverloadWatchdog.kt
                                CoreRecoveryCoordinator.kt
                                CoreRecoveryPolicy.kt
                                CoreRecoveryReceiver.kt
                                CoreRecoveryService.kt
                                Executable.kt
                                GuardedProcessPool.kt
                                LibcoreMemoryPolicy.kt
                                NetworkChangeRecoveryPolicy.kt
                                ProxyService.kt
                                SagerConnection.kt
                                ServiceLifecyclePolicy.kt
                                ServiceNotification.kt
                                SubscriptionBootReceiverPolicy.kt
                                SubscriptionUpdateSchedulePolicy.kt
                                SubscriptionUpdater.kt
                                TileService.kt
                                UrlTestTracker.kt
                                VpnService.kt
                                proto/
                                    BoxInstance.kt
                                    ProfileStatusUpdater.kt
                                    ProxyInstance.kt
                                    TestInstance.kt
                                    TrafficLooper.kt
                                    TrafficLooperPolicy.kt
                                    TrafficUpdater.kt
                                    UrlTest.kt
                            database/
                                CustomDnsServerEntity.kt
                                DataStore.kt
                                GroupManager.kt
                                ImportGroupSelectionPolicy.kt
                                ParcelizeBridge.java
                                ProfileCardPresentation.kt
                                ProfileManager.kt
                                ProfileTransferPolicy.kt
                                ProxyEntity.kt
                                ProxyGroup.kt
                                RuleEntity.kt
                                RuleType.kt
                                SagerDatabase.kt
                                StringCollectionConverter.kt
                                SubscriptionBean.java
                                preference/
                                    EditTextPreferenceModifiers.kt
                                    KeyValuePair.kt
                                    OnPreferenceDataStoreChangeListener.kt
                                    PublicDatabase.kt
                                    RoomPreferenceDataStore.kt
                            dto/
                                IPAPIInfo.kt
                            fmt/
                                AbstractBean.java
                                ConfigBuilder.kt
                                CustomDnsConfigBuilder.kt
                                DirectDnsRouteRuleBuilder.kt
                                DnsEndpointParser.kt
                                DnsServerOptionsBuilder.kt
                                EndpointBootstrapDns.kt
                                KryoConverters.java
                                PluginEntry.kt
                                RouteDnsRuleBuilder.kt
                                Serializable.kt
                                SingBoxSharedOptions.kt
                                TunAddresses.kt
                                TypeMap.kt
                                UniversalFmt.kt
                                gson/
                                    GsonConverters.java
                                http/
                                    HttpBean.java
                                    HttpFmt.kt
                                hysteria/
                                    HysteriaBean.java
                                    HysteriaFmt.kt
                                internal/
                                    ChainBean.java
                                    InternalBean.java
                                    ProxySetBean.java
                                    ProxySetFmt.kt
                                juicity/
                                    JuicityBean.java
                                    JuicityFmt.kt
                                masque/
                                    MasqueBean.java
                                    MasqueFmt.kt
                                masterdns/
                                    MasterDnsVPNBean.java
                                    MasterDnsVPNFmt.kt
                                mieru/
                                    MieruBean.java
                                    MieruFmt.kt
                                naive/
                                    NaiveBean.java
                                    NaiveFmt.kt
                                shadowsocks/
                                    ShadowsocksBean.java
                                    ShadowsocksFmt.kt
                                shadowsocksr/
                                    ShadowsocksRBean.java
                                    ShadowsocksRFmt.kt
                                snell/
                                    SnellBean.java
                                    SnellBuildConfig.kt
                                    SnellFmt.kt
                                socks/
                                    SOCKSBean.java
                                    SOCKSFmt.kt
                                ssh/
                                    SSHBean.java
                                    SSHFmt.kt
                                tailscale/
                                    TailscaleBean.java
                                    TailscaleFmt.kt
                                trojan/
                                    TrojanBean.java
                                    TrojanFmt.kt
                                trojan_go/
                                    TrojanGoBean.java
                                    TrojanGoFmt.kt
                                trusttunnel/
                                    TrustTunnelBean.java
                                    TrustTunnelFmt.kt
                                tuic/
                                    TuicBean.java
                                    TuicFmt.kt
                                v2ray/
                                    StandardV2RayBean.java
                                    V2RayFmt.kt
                                    VMessBean.java
                                    XhttpExtraConverter.kt
                                wireguard/
                                    AmneziaWGBean.java
                                    AmneziaWGFmt.kt
                                    WireGuardBean.java
                                    WireGuardConfParser.kt
                                    WireGuardFmt.kt
                            group/
                                ClashParser.kt
                                GroupInterfaceAdapter.kt
                                GroupUpdater.kt
                                RawUpdater.kt
                                SubscriptionRequestFingerprint.kt
                                XrayParser.kt
                            ktx/
                                AnsiLogFormatter.kt
                                Asyncs.kt
                                Browsers.kt
                                Dialogs.kt
                                Dimens.kt
                                Formats.kt
                                JsonHashNormalizer.kt
                                Kryos.kt
                                Layouts.kt
                                Logs.kt
                                Nets.kt
                                Notification.kt
                                Preferences.kt
                                Utils.kt
                            plugin/
                                PluginManager.kt
                            routing/
                                RoutingImportManager.kt
                                RoutingProfileCodec.kt
                                RoutingProfileExporter.kt
                                RoutingProfileMapper.kt
                                RoutingProfileModels.kt
                                RoutingProviderCatalog.kt
                                RoutingRuleConverters.kt
                                SubscriptionRouting.kt
                            ui/
                                AboutFragment.kt
                                AdblockSettingsActivity.kt
                                AppListActivity.kt
                                AppManagerActivity.kt
                                AssetsActivity.kt
                                BackgroundProcessController.kt
                                BackupFragment.kt
                                BackupImportDialogFragment.kt
                                BlankActivity.kt
                                CellularNetworkActivity.kt
                                ConfigurationFragment.kt
                                ConnectionTestNotificationActionActivity.kt
                                CustomDnsServerSettingsActivity.kt
                                CustomDnsServersActivity.kt
                                CustomThemeFragment.kt
                                GitBackupSettingsActivity.kt
                                GroupConnectionTestController.kt
                                GroupDeletionPolicy.kt
                                GroupFragment.kt
                                GroupPickerActivity.kt
                                GroupSettingsActivity.kt
                                GroupTabSelectionPolicy.kt
                                LegacyMainViewPolicy.kt
                                LogcatFragment.kt
                                LogcatViewModel.kt
                                MainActivity.kt
                                MessageStore.kt
                                NamedFragment.kt
                                NetworkFragment.kt
                                ProfileBatchExport.kt
                                ProfileCardActionPolicy.kt
                                ProfileCardStyling.kt
                                ProfileFileImportPolicy.kt
                                ProfileImportPolicy.kt
                                ProfileSearchPolicy.kt
                                ProfileSelectActivity.kt
                                ProfileSelectionPolicy.kt
                                ProfileSelectionReloadPolicy.kt
                                ProfileShareCapabilities.kt
                                ProfileTcpPingController.kt
                                ProfileUrlTestController.kt
                                QrCodeImageDecoder.kt
                                QrCodeImportParser.kt
                                QuickDisableShortcut.kt
                                QuickEnableShortcut.kt
                                RouteFragment.kt
                                RouteSettingsActivity.kt
                                RoutingImportPreviewActivity.kt
                                RuleAssetNamePolicy.kt
                                RuleSetMatchActivity.kt
                                RuleSetMatchViewModel.kt
                                ScannerActivity.kt
                                SettingsFragment.kt
                                SettingsPreferenceFragment.kt
                                SingBoxConfigPreviewActivity.kt
                                SpeedTestActivity.kt
                                SpeedTestUi.kt
                                StunActivity.kt
                                StunTestModels.kt
                                StunTestViewModel.kt
                                SubscriptionBannerPolicy.kt
                                SubscriptionLinkImportPolicy.kt
                                SwitchActivity.kt
                                ThemedActivity.kt
                                ToolbarFragment.kt
                                ToolbarLayoutActivity.kt
                                ToolsFragment.kt
                                VpnRequestActivity.kt
                                WebDAVSettingsActivity.kt
                                WebviewFragment.kt
                                profile/
                                    AmneziaWGSettingsActivity.kt
                                    ChainSettingsActivity.kt
                                    ConfigEditActivity.kt
                                    HttpSettingsActivity.kt
                                    HysteriaSettingsActivity.kt
                                    JuicitySettingsActivity.kt
                                    MasqueSettingsActivity.kt
                                    MasterDnsVPNSettingsActivity.kt
                                    MieruSettingsActivity.kt
                                    NaiveSettingsActivity.kt
                                    ProfileSettingsActivity.kt
                                    ProxySetSettingsActivity.kt
                                    SSHSettingsActivity.kt
                                    ShadowsocksRSettingsActivity.kt
                                    ShadowsocksSettingsActivity.kt
                                    SnellSettingsActivity.kt
                                    SocksSettingsActivity.kt
                                    StandardV2RaySettingsActivity.kt
                                    TailscaleSettingsActivity.kt
                                    TrojanGoSettingsActivity.kt
                                    TrojanSettingsActivity.kt
                                    TrustTunnelSettingsActivity.kt
                                    TuicSettingsActivity.kt
                                    VMessSettingsActivity.kt
                                    WireGuardSettingsActivity.kt
                                toolbar/
                                    ProfileToolbarActionCatalog.kt
                                    ProfileToolbarLayout.kt
                            utils/
                                AdblockRepository.kt
                                AppCache.kt
                                AppLocale.kt
                                CertificateAuthority.kt
                                Commandline.kt
                                ConnectionIpResolver.kt
                                CrashHandler.kt
                                CustomTheme.kt
                                CustomThemeLink.kt
                                CustomThemePreview.kt
                                DefaultNetworkListener.kt
                                GeoAssetSuggestionRepository.kt
                                PackageCache.kt
                                PhysicalNetworkSelector.kt
                                ProfileCountryResolver.kt
                                RoutingRulesService.kt
                                RulesetSuggestionRepository.kt
                                Subnet.kt
                                SubscriptionTrafficFormatter.kt
                                SubscriptionUserinfo.kt
                                Theme.kt
                            widget/
                                AppIconPreference.kt
                                AppListPreference.kt
                                AutoCollapseTextView.kt
                                CountryBadgeView.kt
                                CountryFlagRenderer.kt
                                DraggableScrollView.kt
                                EndAlignedMarqueeTextView.kt
                                FabCluster.kt
                                GroupPreference.kt
                                LinkOrContentPreference.kt
                                OutboundPreference.kt
                                ProfileListRecyclerView.kt
                                ProfileStatsLayout.kt
                                QRCodeDialog.kt
                                RulesetEditText.kt
                                RulesetEditTextPreferenceDialogFragment.kt
                                ServiceButton.kt
                                StatsBar.kt
                                StatsBarConnectionCheckPolicy.kt
                                StatsBarReconnectPolicy.kt
                                UndoSnackbarManager.kt
                                UserAgentPreference.kt
                                WindowInsetsListeners.kt
                moe/
                    matsuri/
                        nb4a/
                            NativeInterface.kt
                            NativeStateSyncPolicy.kt
                            Protocols.kt
                            SingBoxOptions.java
                            SingBoxOptionsUtil.kt
                            TempDatabase.kt
                            hevtun/
                                HevTunNative.kt
                                HevTunRuntime.kt
                            net/
                                LocalResolverImpl.kt
                            plugin/
                                Plugins.kt
                            proxy/
                                PreferenceBinding.kt
                                PreferenceBindingManager.kt
                                anytls/
                                    AnyTLSBean.java
                                    AnyTLSFmt.kt
                                    AnyTLSSettingsActivity.kt
                                byedpi/
                                    ByeDPIBean.java
                                    ByeDPIFmt.kt
                                    ByeDPISettingsActivity.kt
                                config/
                                    ConfigBean.java
                                    ConfigSettingActivity.kt
                                direct/
                                    DirectBean.java
                                    DirectSettingsActivity.kt
                                neko/
                                    NekoBean.java
                                shadowtls/
                                    ShadowTLSBean.java
                                    ShadowTLSFmt.kt
                                    ShadowTLSSettingsActivity.kt
                            ui/
                                ColorPickerPreference.kt
                                ConnectionTestNotification.kt
                                Dialogs.kt
                                EditConfigPreference.kt
                                ExtendedKeyboard.kt
                                LongClickListPreference.kt
                                LongClickMenuPreference.kt
                                LongClickSwitchPreference.kt
                                MaterialEditTextPreferenceDialogFragment.kt
                                MaterialMultiSelectListPreference.kt
                                MaterialSwitchPreference.kt
                                SimpleMenuPreference.kt
                            utils/
                                HwidGenerator.kt
                                JavaUtil.java
                                KotlinUtil.kt
                                NGUtil.kt
                                SendLog.kt
                                Util.kt
                                WebViewUtil.kt
            res/
                resources.properties
                color/
                    chip_background.xml
                    chip_ripple_color.xml
                    chip_text_color.xml
                    navigation_icon.xml
                    navigation_item.xml
                drawable/
                    baseline_arrow_back_24.xml
                    baseline_construction_24.xml
                    baseline_delete_sweep_24.xml
                    baseline_developer_board_24.xml
                    baseline_flight_takeoff_24.xml
                    baseline_keyboard_tab_24.xml
                    baseline_public_24.xml
                    baseline_redo_24.xml
                    baseline_save_24.xml
                    baseline_send_24.xml
                    baseline_translate_24.xml
                    baseline_undo_24.xml
                    baseline_widgets_24.xml
                    baseline_wrap_text_24.xml
                    bg_dns_rule_badge.xml
                    bg_scanner_frame.xml
                    dialog_background.xml
                    ic_action_copyright.xml
                    ic_action_delete.xml
                    ic_action_description.xml
                    ic_action_dns.xml
                    ic_action_done.xml
                    ic_action_lock.xml
                    ic_action_lock_open.xml
                    ic_action_note_add.xml
                    ic_action_settings.xml
                    ic_app_shortcut_background.xml
                    ic_av_playlist_add.xml
                    ic_baseline_add_24.xml
                    ic_baseline_add_road_24.xml
                    ic_baseline_airplanemode_active_24.xml
                    ic_baseline_android_24.xml
                    ic_baseline_bottom_bar_24.xml
                    ic_baseline_bug_report_24.xml
                    ic_baseline_call_split_24.xml
                    ic_baseline_camera_24.xml
                    ic_baseline_card_giftcard_24.xml
                    ic_baseline_cast_connected_24.xml
                    ic_baseline_center_focus_weak_24.xml
                    ic_baseline_color_lens_24.xml
                    ic_baseline_compare_arrows_24.xml
                    ic_baseline_compress_24.xml
                    ic_baseline_content_copy_24.xml
                    ic_baseline_delete_24.xml
                    ic_baseline_dns_24.xml
                    ic_baseline_domain_24.xml
                    ic_baseline_download_24.xml
                    ic_baseline_emoji_emotions_24.xml
                    ic_baseline_fast_forward_24.xml
                    ic_baseline_fiber_manual_record_24.xml
                    ic_baseline_filter_list_24.xml
                    ic_baseline_fingerprint_24.xml
                    ic_baseline_flip_camera_android_24.xml
                    ic_baseline_folder_open_24.xml
                    ic_baseline_format_align_left_24.xml
                    ic_baseline_grid_3x3_24.xml
                    ic_baseline_home_24.xml
                    ic_baseline_http_24.xml
                    ic_baseline_import_contacts_24.xml
                    ic_baseline_info_24.xml
                    ic_baseline_keyboard_arrow_down_24.xml
                    ic_baseline_layers_24.xml
                    ic_baseline_legend_toggle_24.xml
                    ic_baseline_link_24.xml
                    ic_baseline_local_bar_24.xml
                    ic_baseline_location_on_24.xml
                    ic_baseline_lock_24.xml
                    ic_baseline_low_priority_24.xml
                    ic_baseline_manage_search_24.xml
                    ic_baseline_memory_24.xml
                    ic_baseline_more_vert_24.xml
                    ic_baseline_multiline_chart_24.xml
                    ic_baseline_multiple_stop_24.xml
                    ic_baseline_nat_24.xml
                    ic_baseline_network_ping_24.xml
                    ic_baseline_nfc_24.xml
                    ic_baseline_no_encryption_gmailerrorred_24.xml
                    ic_baseline_notifications_24.xml
                    ic_baseline_notifications_active_24.xml
                    ic_baseline_pause_24.xml
                    ic_baseline_person_24.xml
                    ic_baseline_play_arrow_24.xml
                    ic_baseline_push_pin_24.xml
                    ic_baseline_refresh_24.xml
                    ic_baseline_rule_folder_24.xml
                    ic_baseline_running_with_errors_24.xml
                    ic_baseline_sanitizer_24.xml
                    ic_baseline_save_24.xml
                    ic_baseline_security_24.xml
                    ic_baseline_shuffle_24.xml
                    ic_baseline_shutter_speed_24.xml
                    ic_baseline_speed_24.xml
                    ic_baseline_stream_24.xml
                    ic_baseline_texture_24.xml
                    ic_baseline_timelapse_24.xml
                    ic_baseline_timer_24.xml
                    ic_baseline_transform_24.xml
                    ic_baseline_transgender_24.xml
                    ic_baseline_tune_24.xml
                    ic_baseline_update_24.xml
                    ic_baseline_upload_24.xml
                    ic_baseline_view_list_24.xml
                    ic_baseline_visibility_off_24.xml
                    ic_baseline_vpn_key_24.xml
                    ic_baseline_warning_24.xml
                    ic_baseline_wb_sunny_24.xml
                    ic_communication_phonelink_ring.xml
                    ic_device_data_usage.xml
                    ic_device_developer_mode.xml
                    ic_file_cloud_queue.xml
                    ic_file_file_upload.xml
                    ic_flashlight_24.xml
                    ic_hardware_router.xml
                    ic_image_camera_alt.xml
                    ic_image_edit.xml
                    ic_image_looks_6.xml
                    ic_image_photo.xml
                    ic_launcher_background.xml
                    ic_launcher_monochrome.xml
                    ic_maps_360.xml
                    ic_maps_directions.xml
                    ic_maps_directions_boat.xml
                    ic_navigation_apps.xml
                    ic_navigation_close.xml
                    ic_navigation_menu.xml
                    ic_notification_enhanced_encryption.xml
                    ic_qu_camera_launcher.xml
                    ic_qu_shadowsocks_foreground.xml
                    ic_qu_shadowsocks_launcher.xml
                    ic_service_active.xml
                    ic_service_busy.xml
                    ic_service_connected.xml
                    ic_service_connecting.xml
                    ic_service_idle.xml
                    ic_service_stopped.xml
                    ic_service_stopping.xml
                    ic_settings_password.xml
                    ic_settings_search.xml
                    ic_social_emoji_symbols.xml
                    ic_social_share.xml
                    ic_toolbar_search.xml
                    menu_popup_background.xml
                    splash_screen.xml
                    terminal_scroll_shape.xml
                drawable-v26/
                    ic_qu_camera_launcher.xml
                    ic_qu_shadowsocks_launcher.xml
                font/
                    jetbrains_mono.ttf
                layout/
                    item_dropdown_suggestion.xml
                    item_keyboard_key.xml
                    item_toolbar_action.xml
                    item_toolbar_action_header.xml
                    layout_about.xml
                    layout_add_entity.xml
                    layout_app_list.xml
                    layout_app_placeholder.xml
                    layout_appbar.xml
                    layout_apps.xml
                    layout_apps_item.xml
                    layout_asset_item.xml
                    layout_assets.xml
                    layout_backup.xml
                    layout_cellular_network.xml
                    layout_chain_settings.xml
                    layout_clash_mode_item.xml
                    layout_clash_mode_switch.xml
                    layout_config_settings.xml
                    layout_debug.xml
                    layout_edit_config.xml
                    layout_edit_group.xml
                    layout_empty.xml
                    layout_empty_route.xml
                    layout_git_backup_settings.xml
                    layout_git_branch_name.xml
                    layout_git_compact.xml
                    layout_group.xml
                    layout_group_item.xml
                    layout_group_list.xml
                    layout_group_picker.xml
                    layout_group_picker_item.xml
                    layout_icon_list_item_2.xml
                    layout_import.xml
                    layout_loading.xml
                    layout_local_proxy_dialog.xml
                    layout_logcat.xml
                    layout_loglevel_help.xml
                    layout_main.xml
                    layout_main_drawer_switch.xml
                    layout_network.xml
                    layout_password_dialog.xml
                    layout_profile.xml
                    layout_profile_appbar.xml
                    layout_profile_compact.xml
                    layout_profile_double.xml
                    layout_profile_list.xml
                    layout_progress.xml
                    layout_progress_list.xml
                    layout_proxy_set_settings.xml
                    layout_route.xml
                    layout_route_item.xml
                    layout_routing_export_name_dialog.xml
                    layout_routing_import_preview.xml
                    layout_ruleset_editor.xml
                    layout_ruleset_match.xml
                    layout_ruleset_match_item.xml
                    layout_scanner.xml
                    layout_settings_activity.xml
                    layout_sing_box_config_preview.xml
                    layout_speed_test.xml
                    layout_speed_test_settings.xml
                    layout_stun.xml
                    layout_stun_server_result.xml
                    layout_subscription_banner.xml
                    layout_toolbar_layout.xml
                    layout_tools.xml
                    layout_urltest_preference_dialog.xml
                    layout_webdav_settings.xml
                    layout_webview.xml
                    preference_dialog_material_edittext.xml
                    preference_widget_material_switch.xml
                    simple_menu_dropdown_item.xml
                    widget_adblock_filter_update.xml
                menu/
                    adblock_bundled_filters_menu.xml
                    adblock_custom_filters_menu.xml
                    add_group_menu.xml
                    add_profile_menu.xml
                    add_route_menu.xml
                    app_list_menu.xml
                    double_column_item_menu.xml
                    group_action_menu.xml
                    import_asset_menu.xml
                    logcat_menu.xml
                    main_drawer_menu.xml
                    per_app_proxy_menu.xml
                    profile_apply_menu.xml
                    profile_config_menu.xml
                    profile_selection_menu.xml
                    profile_share_menu.xml
                    scanner_menu.xml
                    settings_menu.xml
                    sing_box_config_preview_menu.xml
                    speed_test_menu.xml
                    toolbar_layout_menu.xml
                    traffic_item_menu.xml
                    traffic_menu.xml
                    yacd_menu.xml
                mipmap-anydpi-v26/
                    ic_launcher.xml
                    ic_launcher_black_white.xml
                    ic_launcher_cyberpunk.xml
                    ic_launcher_dark.xml
                    ic_launcher_druid.xml
                    ic_launcher_halloween.xml
                    ic_launcher_heavens.xml
                    ic_launcher_light.xml
                    ic_launcher_midnight.xml
                    ic_launcher_nekobox.xml
                    ic_launcher_old_nekobox_plus.xml
                    ic_launcher_pink.xml
                    ic_launcher_red.xml
                    ic_launcher_round.xml
                    ic_launcher_russian.xml
                    ic_launcher_text.xml
                mipmap-hdpi/
                    ic_launcher_black_white.webp
                    ic_launcher_black_white_foreground.webp
                    ic_launcher_cyberpunk.webp
                    ic_launcher_cyberpunk_foreground.webp
                    ic_launcher_dark.webp
                    ic_launcher_druid.webp
                    ic_launcher_druid_foreground.webp
                    ic_launcher_foreground.webp
                    ic_launcher_halloween.webp
                    ic_launcher_halloween_foreground.webp
                    ic_launcher_heavens.webp
                    ic_launcher_heavens_foreground.webp
                    ic_launcher_light.webp
                    ic_launcher_midnight.webp
                    ic_launcher_midnight_foreground.webp
                    ic_launcher_nekobox.webp
                    ic_launcher_nekobox_foreground.webp
                    ic_launcher_old_nekobox_plus.webp
                    ic_launcher_old_nekobox_plus_foreground.webp
                    ic_launcher_pink.webp
                    ic_launcher_pink_foreground.webp
                    ic_launcher_red.webp
                    ic_launcher_red_foreground.webp
                    ic_launcher_round.webp
                    ic_launcher_russian.webp
                    ic_launcher_russian_foreground.webp
                    ic_launcher_text.webp
                    ic_launcher_text_foreground.webp
                mipmap-mdpi/
                    ic_launcher_black_white.webp
                    ic_launcher_black_white_foreground.webp
                    ic_launcher_cyberpunk.webp
                    ic_launcher_cyberpunk_foreground.webp
                    ic_launcher_dark.webp
                    ic_launcher_druid.webp
                    ic_launcher_druid_foreground.webp
                    ic_launcher_foreground.webp
                    ic_launcher_halloween.webp
                    ic_launcher_halloween_foreground.webp
                    ic_launcher_heavens.webp
                    ic_launcher_heavens_foreground.webp
                    ic_launcher_light.webp
                    ic_launcher_midnight.webp
                    ic_launcher_midnight_foreground.webp
                    ic_launcher_nekobox.webp
                    ic_launcher_nekobox_foreground.webp
                    ic_launcher_old_nekobox_plus.webp
                    ic_launcher_old_nekobox_plus_foreground.webp
                    ic_launcher_pink.webp
                    ic_launcher_pink_foreground.webp
                    ic_launcher_red.webp
                    ic_launcher_red_foreground.webp
                    ic_launcher_round.webp
                    ic_launcher_russian.webp
                    ic_launcher_russian_foreground.webp
                    ic_launcher_text.webp
                    ic_launcher_text_foreground.webp
                mipmap-night-anydpi-v26/
                    ic_launcher.xml
                    ic_launcher_round.xml
                mipmap-night-hdpi/
                    ic_launcher_round.webp
                mipmap-night-mdpi/
                    ic_launcher_round.webp
                mipmap-night-xhdpi/
                    ic_launcher_round.webp
                mipmap-night-xxhdpi/
                    ic_launcher_round.webp
                mipmap-night-xxxhdpi/
                    ic_launcher_round.webp
                mipmap-xhdpi/
                    ic_launcher_black_white.webp
                    ic_launcher_black_white_foreground.webp
                    ic_launcher_cyberpunk.webp
                    ic_launcher_cyberpunk_foreground.webp
                    ic_launcher_dark.webp
                    ic_launcher_druid.webp
                    ic_launcher_druid_foreground.webp
                    ic_launcher_foreground.webp
                    ic_launcher_halloween.webp
                    ic_launcher_halloween_foreground.webp
                    ic_launcher_heavens.webp
                    ic_launcher_heavens_foreground.webp
                    ic_launcher_light.webp
                    ic_launcher_midnight.webp
                    ic_launcher_midnight_foreground.webp
                    ic_launcher_nekobox.webp
                    ic_launcher_nekobox_foreground.webp
                    ic_launcher_old_nekobox_plus.webp
                    ic_launcher_old_nekobox_plus_foreground.webp
                    ic_launcher_pink.webp
                    ic_launcher_pink_foreground.webp
                    ic_launcher_red.webp
                    ic_launcher_red_foreground.webp
                    ic_launcher_round.webp
                    ic_launcher_russian.webp
                    ic_launcher_russian_foreground.webp
                    ic_launcher_text.webp
                    ic_launcher_text_foreground.webp
                mipmap-xxhdpi/
                    ic_launcher_black_white.webp
                    ic_launcher_black_white_foreground.webp
                    ic_launcher_cyberpunk.webp
                    ic_launcher_cyberpunk_foreground.webp
                    ic_launcher_dark.webp
                    ic_launcher_druid.webp
                    ic_launcher_druid_foreground.webp
                    ic_launcher_foreground.webp
                    ic_launcher_halloween.webp
                    ic_launcher_halloween_foreground.webp
                    ic_launcher_heavens.webp
                    ic_launcher_heavens_foreground.webp
                    ic_launcher_light.webp
                    ic_launcher_midnight.webp
                    ic_launcher_midnight_foreground.webp
                    ic_launcher_nekobox.webp
                    ic_launcher_nekobox_foreground.webp
                    ic_launcher_old_nekobox_plus.webp
                    ic_launcher_old_nekobox_plus_foreground.webp
                    ic_launcher_pink.webp
                    ic_launcher_pink_foreground.webp
                    ic_launcher_red.webp
                    ic_launcher_red_foreground.webp
                    ic_launcher_round.webp
                    ic_launcher_russian.webp
                    ic_launcher_russian_foreground.webp
                    ic_launcher_text.webp
                    ic_launcher_text_foreground.webp
                mipmap-xxxhdpi/
                    ic_launcher_black_white.webp
                    ic_launcher_black_white_foreground.webp
                    ic_launcher_cyberpunk.webp
                    ic_launcher_cyberpunk_foreground.webp
                    ic_launcher_dark.webp
                    ic_launcher_druid.webp
                    ic_launcher_druid_foreground.webp
                    ic_launcher_foreground.webp
                    ic_launcher_halloween.webp
                    ic_launcher_halloween_foreground.webp
                    ic_launcher_heavens.webp
                    ic_launcher_heavens_foreground.webp
                    ic_launcher_light.webp
                    ic_launcher_midnight.webp
                    ic_launcher_midnight_foreground.webp
                    ic_launcher_nekobox.webp
                    ic_launcher_nekobox_foreground.webp
                    ic_launcher_old_nekobox_plus.webp
                    ic_launcher_old_nekobox_plus_foreground.webp
                    ic_launcher_pink.webp
                    ic_launcher_pink_foreground.webp
                    ic_launcher_red.webp
                    ic_launcher_red_foreground.webp
                    ic_launcher_round.webp
                    ic_launcher_russian.webp
                    ic_launcher_russian_foreground.webp
                    ic_launcher_text.webp
                    ic_launcher_text_foreground.webp
                raw/
                    insecure.txt
                    not_encrypted.txt
                    shadowsocks_stream_cipher.txt
                    vmess_md5_auth.txt
                raw-zh-rCN/
                    insecure.txt
                    not_encrypted.txt
                    shadowsocks_stream_cipher.txt
                    vmess_md5_auth.txt
                values/
                    arrays.xml
                    attrs.xml
                    bools.xml
                    colors.xml
                    dimens.xml
                    ic_launcher_background.xml
                    strings.xml
                    themes.xml
                values-ar/
                    arrays.xml
                    strings.xml
                values-be/
                    arrays.xml
                    strings.xml
                values-de/
                    arrays.xml
                    strings.xml
                values-es/
                    arrays.xml
                    strings.xml
                values-fa/
                    arrays.xml
                    strings.xml
                values-fr/
                    arrays.xml
                    strings.xml
                values-in/
                    arrays.xml
                    strings.xml
                values-it/
                    arrays.xml
                    strings.xml
                values-ja/
                    arrays.xml
                    strings.xml
                values-ko/
                    arrays.xml
                    strings.xml
                values-nb-rNO/
                    arrays.xml
                    strings.xml
                values-night/
                    colors.xml
                    ic_launcher_background.xml
                values-nl/
                    arrays.xml
                    strings.xml
                values-pt-rBR/
                    arrays.xml
                    strings.xml
                values-ru/
                    arrays.xml
                    strings.xml
                values-tr/
                    arrays.xml
                    strings.xml
                values-uk/
                    arrays.xml
                    strings.xml
                values-v28/
                    bools.xml
                values-v31/
                    themes.xml
                values-zh-rCN/
                    arrays.xml
                    strings.xml
                values-zh-rHK/
                    arrays.xml
                    strings.xml
                values-zh-rTW/
                    arrays.xml
                    strings.xml
                xml/
                    adblock_preferences.xml
                    amneziawg_preferences.xml
                    anytls_preferences.xml
                    backup_descriptor.xml
                    backup_rules.xml
                    balancer_preferences.xml
                    byedpi_preferences.xml
                    cache_paths.xml
                    config_preferences.xml
                    direct_preferences.xml
                    global_preferences.xml
                    group_preferences.xml
                    hysteria_preferences.xml
                    juicity_preferences.xml
                    masque_preferences.xml
                    masterdnsvpn_preferences.xml
                    mieru_preferences.xml
                    naive_preferences.xml
                    name_preferences.xml
                    neko_preferences.xml
                    network_security_config.xml
                    proxy_set_preferences.xml
                    route_preferences.xml
                    shadowsocks_preferences.xml
                    shadowsocksr_preferences.xml
                    shadowtls_preferences.xml
                    shortcuts.xml
                    snell_preferences.xml
                    socks_preferences.xml
                    ssh_preferences.xml
                    standard_v2ray_preferences.xml
                    tailscale_preferences.xml
                    trojan_go_preferences.xml
                    trusttunnel_preferences.xml
                    tuic_preferences.xml
                    webdav_preferences.xml
                    wireguard_preferences.xml
        test/
            java/
                io/
                    nekohasekai/
                        sagernet/
                            AndroidTunPayloadTest.kt
                            AppIconStatePolicyTest.kt
                            AppLogLevelTest.kt
                            AppVersionCachePolicyTest.kt
                            BatteryOptimizationTest.kt
                            BootReceiverPolicyTest.kt
                            LocalNetworkPermissionTest.kt
                            LogcatRetentionSizeTest.kt
                            backup/
                                BackupContainerCodecTest.kt
                                GitBackupConfigValidatorTest.kt
                                GitBackupErrorClassifierTest.kt
                                GitBackupRepositoryTest.kt
                            bg/
                                AutomaticConnectionTestPolicyTest.kt
                                ConnectionRecoveryPolicyTest.kt
                                ConnectionTestSessionStateTest.kt
                                CoreOverloadDetectorTest.kt
                                CoreRecoveryPolicyTest.kt
                                LibcoreMemoryPolicyTest.kt
                                NetworkChangeRecoveryPolicyTest.kt
                                ServiceJobCancellationTest.kt
                                ServiceLifecyclePolicyTest.kt
                                SubscriptionBootReceiverPolicyTest.kt
                                SubscriptionUpdateSchedulePolicyTest.kt
                                UrlTestTrackerTest.kt
                                proto/
                                    TrafficLooperPolicyTest.kt
                                    UrlTestProfileEligibilityTest.kt
                            database/
                                ImportGroupSelectionPolicyTest.kt
                                ProfileCardPresentationTest.kt
                                ProfileTransferPolicyTest.kt
                                ProxyCountryPersistenceTest.kt
                                ProxyEntityBeanTest.kt
                                ProxyEntityGroupExportTest.kt
                            fmt/
                                AbstractBeanSerializationTest.kt
                                BypassLanRouteAddressTest.kt
                                ConfigBuilderGlobalOutboundTest.kt
                                CustomDnsConfigBuilderTest.kt
                                DirectDnsRouteRuleBuilderTest.kt
                                DnsGlobalOptionsTest.kt
                                DnsServerOptionsBuilderTest.kt
                                GroupMuxResolutionTest.kt
                                RouteClashModeRuleTest.kt
                                RouteDnsRuleBuilderTest.kt
                                SingBoxSharedOptionsTest.kt
                                TrafficFragmentationEligibilityTest.kt
                                TunSystemDnsRouteRulesTest.kt
                                TunUnrecognizedTrafficRuleTest.kt
                                internal/
                                    ProxySetFmtTest.kt
                                masque/
                                    MasqueFmtTest.kt
                                mieru/
                                    MieruFmtTest.kt
                                shadowsocks/
                                    ShadowsocksFmtTest.kt
                                snell/
                                    SnellFmtTest.kt
                                ssh/
                                    SSHFmtTest.kt
                                tailscale/
                                    TailscaleFmtTest.kt
                                trusttunnel/
                                    TrustTunnelFmtTest.kt
                                v2ray/
                                    ClashXhttpImportTest.kt
                                    V2RayFmtTest.kt
                                    VlessPacketEncodingTest.kt
                                    XhttpLinkFormatTest.kt
                                wireguard/
                                    AmneziaWGFmtTest.kt
                                    WireGuardConfParserTest.kt
                                    WireGuardLinkFormatTest.kt
                            group/
                                ClashAwgMasqueImportTest.kt
                                ClashParserTest.kt
                                RawUpdaterMuxSettingsTest.kt
                                RawUpdaterTest.kt
                                SubscriptionRequestFingerprintTest.kt
                                WireGuardConfImportTest.kt
                                XrayParserTest.kt
                            ktx/
                                AmneziaVpnImportTest.kt
                                AnsiLogFormatterTest.kt
                            routing/
                                RoutingProfileCodecTest.kt
                                RoutingProfileExporterPolicyTest.kt
                                RoutingRuleConvertersTest.kt
                                SubscriptionRoutingTest.kt
                            ui/
                                CustomThemeDeepLinkManifestTest.kt
                                GroupConnectionTestUrlTest.kt
                                GroupDeletionPolicyTest.kt
                                GroupTabSelectionPolicyTest.kt
                                LegacyMainViewPolicyTest.kt
                                LogcatPolicyTest.kt
                                ProfileBatchExportTest.kt
                                ProfileCardActionPolicyTest.kt
                                ProfileDeepLinkManifestTest.kt
                                ProfileFileImportPolicyTest.kt
                                ProfileImportPolicyTest.kt
                                ProfileSearchPolicyTest.kt
                                ProfileSelectionPolicyTest.kt
                                ProfileSelectionReloadPolicyTest.kt
                                ProfileShareCapabilitiesTest.kt
                                QrCodeImageDecoderTest.kt
                                QrCodeImportParserTest.kt
                                RoutingDeepLinkManifestTest.kt
                                RuleAssetNamePolicyTest.kt
                                SpeedTestUiTest.kt
                                StunTestModelsTest.kt
                                SubscriptionBannerPolicyTest.kt
                                SubscriptionLinkImportPolicyTest.kt
                                toolbar/
                                    ProfileToolbarActionCatalogTest.kt
                                    ProfileToolbarLayoutTest.kt
                            utils/
                                AppCacheTest.kt
                                ConnectionIpResolverTest.kt
                                CustomThemeLinkTest.kt
                                CustomThemePreviewTest.kt
                                PhysicalNetworkSelectorTest.kt
                                ProfileCountryResolverTest.kt
                                SubscriptionTrafficFormatterTest.kt
                            widget/
                                CountryFlagAssetsTest.kt
                                ProfileCardMarqueeLayoutTest.kt
                                StatsBarConnectionCheckPolicyTest.kt
                                StatsBarReconnectPolicyTest.kt
                moe/
                    matsuri/
                        nb4a/
                            NativeStateSyncPolicyTest.kt
                            ProtocolsConnectionTestErrorTest.kt
                            ProtocolsDeduplicationTest.kt
                            proxy/
                                direct/
                                    DirectBeanTest.kt
                                shadowtls/
                                    ShadowTLSFmtTest.kt
                            utils/
                                HwidGeneratorTest.kt
`

---

## 2. Пофайловый разбор компонентов

### 2.1. Конфигурация сборки и среды
- **[`build.gradle.kts`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/build.gradle.kts)**:
  - Основной скрипт сборки модуля `:app`.
  - Включает плагины `com.android.application`, `ksp`, `parcelize`.
  - Регистрирует задачу `validateBundledSingBoxAssets`, проверяющую наличие и целостность критичных бандлированных ресурсов (`geoip.db.xz`, `geosite.db.xz`, `throne-ruleset-srslist.h`, `itdog-ruleset.json`).
  - Задача `buildHevTun` компилирует нативную библиотеку `libhev-socks5-tunnel.so` под архитектуры `armeabi-v7a`, `arm64-v8a`, `x86`, `x86_64`.
  - Подключает зависимости: Room 2.8.4 с Roomigrant, CameraX + ZXing (сканирование QR-кодов), JGit (синхронизация бэкапов через Git), Editorkit (редактор JSON/конфигов), Material Design 1.13, desugar_jdk_libs.

- **[`proguard-rules.pro`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/proguard-rules.pro)**:
  - Правила минификации и обфускации ProGuard/R8.
  - Сохраняет аннотации Parcelize, сериализаторы Kryo, модели Gson/Room и нативные JNI экспорты (`go.Seq`, `libcore.*`, `moe.matsuri.nb4a.*`).

- **[`AndroidManifest.xml`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/AndroidManifest.xml)**:
  - Декларирует разрешения: `VpnService`, `FOREGROUND_SERVICE`, `INTERNET`, `RECEIVE_BOOT_COMPLETED`, `QUERY_ALL_PACKAGES` (для раздельного туннелирования по приложениям).
  - Определяет мультипроцессную модель: основной процесс UI (`io.nekohasekai.sagernet`) и выделенный фоновый процесс для сервиса туннеля и ядра (`:bg`).
  - Регистрирует активности, службы (`VpnService`, `ProxyService`, `TileService`), ресиверы и шорткаты (`QuickToggleShortcut`).

---

### 2.2. Корневые классы приложения (`io.nekohasekai.sagernet.*`)

#### 1. [`SagerNet.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/SagerNet.kt)
- **Роль:** Главный класс приложения (`Application`), точка входа жизненного цикла Android-процесса.
- **Архитектура и логика:**
  - Реализует `WorkConfiguration.Provider` для настройки фонового WorkManager в процессе `:bg`.
  - Разделяет инициализацию между основным процессом UI и фоновым процессом ядра `:bg`.
  - В методе `onCreate()`:
    - Устанавливает глобальный обработчик крашей `CrashHandler`.
    - Проверяет обновление версии приложения и при необходимости очищает кэш через `AppVersionCachePolicy`.
    - Инициализирует Go runtime (`Seq.setContext`) и прокси-ядро через `Libcore.initCore()` с путями к кэшу, ассетам, буферам логов и JNI-интерфейсам `NativeInterface` и `LocalResolverImpl`.
    - Для UI-процесса применяет динамические темы Material You (`DynamicColors`), ночную тему, системный язык `AppLocale` и каналы уведомлений (`updateNotificationChannels`).
    - Запускает `DefaultNetworkListener` для отслеживания активной сети и синхронизации ее состояния с Go-ядром (`nativeInterface.syncNetworkState`).
  - Предоставляет статические функции управления сервисом:
    - `startService()`: отправляет `Intent` на запуск службы `VpnService` / `ProxyService` через `SagerConnection.serviceClass` с текущим активным профилем.
    - `reloadService(profileId)`: отправляет команду перезагрузки ядра без полного пересоздания сервиса.
    - `stopService()`: отправляет broadcast `Action.CLOSE` на отключение туннеля.
    - `scheduleLibcoreGCSweep()`: асинхронный вызов сборщика мусора Go ядра при нехватке памяти (`onTrimMemory`).

#### 2. [`Constants.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/Constants.kt)
- **Роль:** Центральный репозиторий констант всего Android-приложения.
- **Содержимое:**
  - `CONNECTION_TEST_URL`: дефолтный URL проверки подключения (`https://www.gstatic.com/generate_204`).
  - `Key`: ключи SharedPreferences/DataStore (настройки темы, DNS, FakeDNS, обхода LAN, TUN, AdBlock, Mux, портов смешанного прокси 2080).
  - `Action`: системные Intent actions (`START`, `RELOAD`, `CLOSE`, `UPDATE_STATUS`).

#### 3. [`AndroidTunPayload.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/AndroidTunPayload.kt)
- **Роль:** Стабильный версионированный DTO-контракт между Go-ядром (`libcore`) и Android `VpnService`.
- **Логика работы:**
  - Принимает JSON от Go-метода `libcore.BoxPlatformInterface.OpenTun`.
  - Парсит и валидирует параметры TUN: MTU (от 576 до 65535), семейство адресов IPv4/IPv6, CIDR-маски, in-TUN DNS серверы и списки маршрутов.
  - Преобразует сырые данные в типобезопасный `Plan`, готовый для передачи в `VpnService.Builder`. Содержит собственные JVM-тестируемые парсеры CIDR/IP без зависимостей от Android SDK.

#### 4. [`AppIconManager.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/AppIconManager.kt)
- **Роль:** Динамическое переключение иконки приложения на лаунчере.
- **Логика работы:**
  - Содержит перечисление 15 стилей иконок (`AppIcon`: `NEKOBOX_PLUS`, `LIGHT_MODE`, `DARK_MODE`, `MIDNIGHT`, `CYBERPUNK`, `RUSSIAN` и др.).
  - Использует механизм `activity-alias` в Android PackageManager: включает выбранный алиас и отключает остальные с флагом `DONT_KILL_APP`.
  - Поддерживает предварительный просмотр темы иконки с учетом ночного режима системы.

#### 5. [`AppLogLevel.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/AppLogLevel.kt)
- **Роль:** Управление уровнем логирования ядра и приложения.
- **Логика работы:**
  - Сопоставляет числовые значения настроек Android с текстовыми уровнями `sing-box` (`warn`, `info`, `debug`, `trace`, `panic`, `fatal`, `error`).
  - Предоставляет потокобезопасный синглтон `AppLogLevelController` для фильтрации сообщений лога по приоритету.

#### 6. [`AppVersionCachePolicy.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/AppVersionCachePolicy.kt)
- **Роль:** Чистая логика проверки необходимости сброса кэша.
- **Логика работы:**
  - Функция `shouldClearCache`: сравнивает сохраненный `versionCode` с текущим скомпилированным кодом версии, инициируя инвалидацию дискового кэша после обновлений.

#### 7. [`BatteryOptimization.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/BatteryOptimization.kt)
- **Роль:** Проверка и запрос исключения из режима оптимизации батареи Doze Mode.
- **Логика работы:**
  - Формирует системный Intent `ACTION_REQUEST_IGNORE_BATTERY_OPTIMIZATIONS` для обеспечения бесперебойной работы VPN-сервиса в фоновом режиме.

#### 8. [`BootReceiver.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/BootReceiver.kt)
- **Роль:** `BroadcastReceiver` системных событий завершения загрузки устройства (`BOOT_COMPLETED`, `LOCKED_BOOT_COMPLETED`).
- **Логика работы:**
  - Вызывает переконфигурацию расписания обновления подписок `SubscriptionUpdater.reconfigureUpdater()`.
  - Проверяет через `BootReceiverPolicy` флаг автоподключения `persistAcrossReboot` и запускает VPN-сервис при старте системы.

#### 9. [`BootReceiverPolicy.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/BootReceiverPolicy.kt)
- **Роль:** Изолированная бизнес-логика автозапуска на загрузке устройства.
- **Логика работы:**
  - Проверяет флаг `persistAcrossReboot`, валидность выбранного прокси (`selectedProxy > 0`) и доступность расшифрованного хранилища пользователя (`isUserUnlocked` для режима Direct Boot).

#### 10. [`LocalNetworkPermission.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/LocalNetworkPermission.kt)
- **Роль:** Валидация разрешений локальной сети для новых версий Android.
- **Логика работы:**
  - Проверяет наличие `android.permission.ACCESS_LOCAL_NETWORK` (Android 16+) при использовании системного или смешанного TUN-стека (`SYSTEM` / `MIXED`).

#### 11. [`LogcatRetentionSize.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/LogcatRetentionSize.kt)
- **Роль:** Парсер и валидатор размера кольцевого буфера логов logcat.
- **Логика работы:**
  - Парсит строки вида `250kb`, `10mb`, нормализует единицы измерения, ограничивает диапазон от 10 КБ до 1024 МБ.

#### 12. [`QuickToggleShortcut.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/QuickToggleShortcut.kt)
- **Роль:** Activity-шорткат для быстрого включения/отключения VPN с домашнего экрана Android.
- **Логика работы:**
  - Подключается к службе через `SagerConnection`.
  - Если сервис запущен — останавливает его либо переключает на переданный в шорткате профиль.
  - Если сервис остановлен — запускает его.

---

### 2.3. Пакет межпроцессного взаимодействия (`io.nekohasekai.sagernet.aidl.*`)

- **[`SpeedDisplayData.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/aidl/SpeedDisplayData.kt)**:
  - Parcelable DTO модель для передачи текущей скорости трафика в реальном времени из фонового процесса ядра `:bg` в процесс UI.
  - Поля: `txRateProxy` / `rxRateProxy` (байт/с через прокси), `txRateDirect` / `rxRateDirect` (прямой трафик), `txTotal` / `rxTotal` (накопленный объем за текущую сессию).
- **[`SpeedTestData.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/aidl/SpeedTestData.kt)**:
  - Parcelable DTO для трансляции состояния и фаз замера скорости (Speedtest).
  - Фазы: `PHASE_IDLE`, `PHASE_FINDING_SERVER`, `PHASE_DOWNLOAD`, `PHASE_UPLOAD`, `PHASE_COMPLETE`, `PHASE_ERROR`, `PHASE_CANCELLED`.
  - Содержит метрики: процент прогресса, задержка (мс), скорости отдачи/загрузки, метаданные сервера (имя, страна).
- **[`TrafficData.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/aidl/TrafficData.kt)**:
  - Parcelable модель учета входящего/исходящего трафика по конкретному идентификатору профиля (`id`, `tx`, `rx`).
- **[`TrafficDataBatch.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/aidl/TrafficDataBatch.kt)**:
  - Батч-обертка для пакетной передачи массива `TrafficData` через IPC транзакцию Android Binder для снижения накладных расходов.

---

### 2.4. Пакет резервного копирования и синхронизации (`io.nekohasekai.sagernet.backup.*`)

- **[`BackupContainerCodec.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/backup/BackupContainerCodec.kt)**:
  - Криптографический кодек безопасного контейнера бэкапа (`NBPLUSBAK`).
  - Применяет PBKDF2-HMAC-SHA256 (120 000 итераций, 16-байтный случайный salt) для деривации 256-битного ключа из пароля.
  - Шифрование данных: алгоритм `AES/GCM/NoPadding` (12-байтный случайный nonce, 128-битный аутентификационный тег).
  - Префикс заголовка и метаданные защищаются через AAD (`Additional Authenticated Data`). При неверном пароле или повреждении файла выбрасывает `BackupPasswordException`.
- **[`GitBackupModels.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/backup/GitBackupModels.kt)**:
  - DTO структуры `GitBackupConfig` (HTTPS URL репозитория, логин, ветка, токен доступа, пароль шифрования) и `GitRestorePoint` (коммит, время, сообщение, статус верификации).
  - Валидатор `GitBackupConfigValidator`: проверяет корректность HTTPS URL (запрет внедрения паролей в URL, проверка хоста) и имени ветки Git.
- **[`GitBackupRepository.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/backup/GitBackupRepository.kt)**:
  - Движок Git-синхронизации на базе библиотеки JGit (`org.eclipse.jgit`).
  - Позволяет клонировать/синхронизировать удаленные репозитории (GitHub, GitLab, самописные Git-серверы), создавать ветку бэкапа, коммитить зашифрованный бинарный контейнер `nekobox_plus_backup.bin` и пушить изменения.
  - Поддерживает просмотр истории точек восстановления и скачивание любого исторического коммита для отката настроек.
- **[`GitBackupConfigStore.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/backup/GitBackupConfigStore.kt)**:
  - Хранилище настроек Git-бэкапа в DataStore / EncryptedSharedPreferences.
- **[`GitBackupErrorClassifier.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/backup/GitBackupErrorClassifier.kt)**:
  - Классификатор ошибок JGit (ошибки авторизации 401/403, таймауты сети, конфликты веток, недоступность хоста).
- **[`GitHttpConnectionFactory.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/backup/GitHttpConnectionFactory.kt)**:
  - Фабрика HTTP-соединений для JGit с кастомными таймаутами и системным User-Agent (`git/2.55.0`).

---

### 2.5. Пакет фоновых служб и ядра (`io.nekohasekai.sagernet.bg.*`)

- **[`BaseService.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/bg/BaseService.kt)**:
  - Главный координатор жизненного цикла сервиса проксирования.
  - Содержит автомат состояний `State`: `Idle`, `Connecting`, `Connected`, `Stopping`, `Stopped`.
  - Отслеживает сбои нативного ядра (`LibcoreCrashType`: Go panic, SIGSEGV, heap corruption) и координирует автоматическое восстановление (`CoreRecoveryCoordinator`).
  - Управляет сторожевым таймером перегрузки `CoreOverloadWatchdog` и фоновыми задержками сети (`ConnectionRecoveryPolicy`).
  - Реализует серверный AIDL интерфейс `ISagerNetService` для обработки команд из UI (старт, стоп, перезагрузка, получение трафика и логов).
- **[`VpnService.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/bg/VpnService.kt)**:
  - Наследник `android.net.VpnService` и реализация `BaseService.Interface`.
  - Управляет созданием виртуального сетевого интерфейса Android TUN (`ParcelFileDescriptor`), сокетами защиты `protect()` (чтобы трафик ядра не зацикливался), интеграцией с `HevSocks5Tunnel` (при включении `enableHevTun`) и частичным WakeLock (`sagernet:vpn`).
- **[`ProxyService.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/bg/ProxyService.kt)**:
  - Легковесный сервис для режима локального прокси (без создания системного VPN-адаптера), запускающий только входящие порты SOCKS/HTTP.
- **[`SagerConnection.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/bg/SagerConnection.kt)**:
  - Клиентская реализация `ServiceConnection` для связи Activities и ViewModels с фоновым процессом `:bg`.
  - Подписывается на коллбэки `ISagerNetServiceCallback`, транслируя события изменения статуса подключения и скорости в UI-поток.
- **[`TileService.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/bg/TileService.kt)**:
  - Плитка быстрых настроек Android Quick Settings для переключения VPN из шторки уведомлений.
- **[`SubscriptionUpdater.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/bg/SubscriptionUpdater.kt)**:
  - Фоновый воркер `CoroutineWorker` на базе AndroidX WorkManager для автоматического обновления подписок по расписанию (`auto_update_minutes`).
- **[`proto/BoxInstance.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/bg/proto/BoxInstance.kt)**:
  - Менеджер экземпляра sing-box в процессе `:bg`.
  - Вызывает `Libcore.newSingBoxInstance(configJson)`, управляет его запуском, graceful-остановкой, собирает метрики памяти и транслирует аварии.
- **[`proto/TrafficLooper.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/bg/proto/TrafficLooper.kt)** и **[`TrafficUpdater.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/bg/proto/TrafficUpdater.kt)**:
  - Высокочастотный опрос счетчиков байт ядра Go, обновление статистики профилей в базе данных Room и оповещение UI-подписчиков.
- **[`proto/UrlTest.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/bg/proto/UrlTest.kt)**:
  - Выполнение замеров задержек (TCP Ping, HTTP 204) по активным и фоновым профилям.

---

### 2.6. Пакет базы данных и сохранения состояния (`io.nekohasekai.sagernet.database.*`)

- **[`SagerDatabase.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/database/SagerDatabase.kt)**:
  - Основная Room Database приложения (версия схемы 27).
  - Управляет сущностями: `ProxyEntity` (прокси-серверы), `ProxyGroup` (группы/подписки), `RuleEntity` (правила маршрутизации).
  - Использует сериализаторы `KryoConverters` и `GsonConverters` для упаковки сложных объектов конфигураций в BLOB/JSON поля.
  - Содержит миграции базы данных, включая автомиграции и исправление списков доменов для России (`RUSSIA_DOMAIN_RULE_NAMES`).
- **[`ProxyEntity.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/database/ProxyEntity.kt)**:
  - Центральная модель данных профиля прокси.
  - Содержит метаданные: ID, ID группы, тип протокола, статистика трафика (tx/rx), пинг, код страны (GeoIP), статус.
  - Инкапсулирует более 25 специфичных протокольных бинов (`vmessBean`, `vlessBean`, `trojanBean`, `ssBean`, `hysteriaBean`, `wireGuardBean`, `amneziaWGBean`, `tuicBean`, `juicityBean`, `byeDPIBean`, `masterDnsVPNBean`, `chainBean`, `proxySetBean` и др.).
- **[`DataStore.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/database/DataStore.kt)**:
  - Высокоуровневый фасад доступа к глобальным настройкам приложения поверх `RoomPreferenceDataStore`.
- **[`GroupManager.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/database/GroupManager.kt)**:
  - Управление CRUD операциями над группами профилей и подписками.
- **[`ProfileManager.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/database/ProfileManager.kt)**:
  - Управление жизненным циклом профилей: импорт, экспорт, сортировка, дублирование, удаление, сохранение результатов тестирования скорости и задержки.

---

### 2.7. Форматирование и сборка конфигураций ядра (`io.nekohasekai.sagernet.fmt.*`)

- **[`ConfigBuilder.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/fmt/ConfigBuilder.kt)**:
  - Ключевой и крупнейший генератор конфигураций ядра `sing-box` (2500+ строк кода).
  - **Inbounds:** Конструирует входящие интерфейсы — `tun-in` (с адресами TUN, MTU, авто-маршрутизацией), `mixed-in` (SOCKS/HTTP порт 2080), DNS-in.
  - **Outbounds:** Транслирует сущности `ProxyEntity` во внутренние JSON-структуры sing-box с поддержкой TLS, Reality, uTLS, multiplex (smux/yamux), TCP Brutal, ECH, кастомных заголовков.
  - **Служебные Outbounds:** Формирует узлы `direct`, `bypass`, `block`, `dns-out`, цепочки `fragment` (обход блокировок по TLS ClientHello) и `byedpi-fragment`.
  - **DNS:** Строит блоки `remote-dns`, `direct-dns`, FakeDNS (`198.18.0.0/15`), правила разрешения имен и DNS over HTTPS/TLS/QUIC.
  - **Routing:** Генерирует массив правил `route.rules` на основе настроек маршрутизации, раздельного туннелирования приложений (`proxyApps` / `bypassMode`), гео-баз GeoIP / Geosite и списков доменов.
  - **Clash API:** Формирует секцию `experimental.clash_api` для внешнего управления и интеграции с веб-панелями.
- **[`CustomDnsConfigBuilder.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/fmt/CustomDnsConfigBuilder.kt)**:
  - Сборка кастомных серверов DNS (DoH, DoT, DoQ, Plain DNS) и правил резолвинга.
- **[`SingBoxSharedOptions.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/fmt/SingBoxSharedOptions.kt)**:
  - Общие опции мультиплексирования, TCP Fast Open, Brutal congestion control.
- **Подпакеты протокольных сериализаторов:**
  - `wireguard/AmneziaWGFmt.kt`, `wireguard/WireGuardFmt.kt`: Сборка AmneziaWG (с обфускацией параметров Jc, Jmin, Jmax, S1, S2, H1-H4) и классического WireGuard.
  - `v2ray/V2RayFmt.kt`: Сборка VMess / VLESS с транспортными протоколами gRPC, WebSocket, HTTP/2, XHTTP (Splithttp).
  - `shadowsocks/ShadowsocksFmt.kt`: Сборка Shadowsocks с плагинами v2ray-plugin, obfs, shadow-tls.
  - `hysteria/HysteriaFmt.kt`: Сборка Hysteria v1 и Hysteria v2.
  - `trusttunnel/TrustTunnelFmt.kt`, `juicity/JuicityFmt.kt`, `masque/MasqueFmt.kt`: Поддержка кастомных протоколов обхода.

---

### 2.8. Пакет парсинга подписок и групп (`io.nekohasekai.sagernet.group.*`)

- **[`RawUpdater.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/group/RawUpdater.kt)**:
  - Центральный движок скачивания и парсинга подписок.
  - Сетевая загрузка через `libcore.HTTPRequest` с поддержкой спуфинга клиентов (`SpoofApp`: v2rayN, Clash, Shadowrocket, NekoBox) и uTLS-фингерпринтов (`SubscriptionRequestFingerprint`).
  - Парсинг Base64-данных, списков ссылок (`vless://`, `vmess://`, `trojan://`, `ss://`, `hysteria2://`, `wireguard://`, `awg://`), JSON-списков и WireGuard `.conf` файлов.
  - Чтение сервисных заголовков `Subscription-Userinfo` (лимит трафика, остаток, дата истечения срока) и `Profile-Update-Interval`.
  - Дедупликация серверов, фильтрация по регулярным выражениям (`filterRegex`) и автоопределение страны сервера через `ProfileCountryResolver`.
- **[`ClashParser.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/group/ClashParser.kt)**:
  - Парсер YAML-конфигов Clash/Mihomo для извлечения массива `proxies` в сущности `ProxyEntity`.
- **[`XrayParser.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/group/XrayParser.kt)**:
  - Парсер JSON-конфигураций Xray (VLESS Reality, Vision, gRPC, WebSocket).

---

### 2.9. Пакет маршрутизации (`io.nekohasekai.sagernet.routing.*`)

- **[`RoutingProfileModels.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/routing/RoutingProfileModels.kt)** и **[`RoutingProfileCodec.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/routing/RoutingProfileCodec.kt)**:
  - Экспорт и импорт наборов правил маршрутизации (JSON/ZIP профили).
- **[`RoutingRuleConverters.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/routing/RoutingRuleConverters.kt)**:
  - Конвертация правил из UI-сущностей `RuleEntity` в sing-box формат правил (domain, domain_suffix, ip_cidr, geoip, geosite, port, process_name).
- **[`SubscriptionRouting.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/routing/SubscriptionRouting.kt)**:
  - Автоматическое извлечение и применение правил маршрутизации, поставляемых напрямую в подписках.

---

### 2.10. Пользовательский интерфейс (`io.nekohasekai.sagernet.ui.*`)

- **[`MainActivity.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/ui/MainActivity.kt)**:
  - Главный экран приложения. Содержит вкладки групп/подписок (`GroupFragment`), плавающую кнопку запуска (FAB) VPN, панель статистики трафика в реальном времени (`StatsBar`) и доступ к настройкам.
- **[`SettingsPreferenceFragment.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/ui/SettingsPreferenceFragment.kt)**:
  - Главный экран глобальных настроек приложения (разделы: Подключение, DNS, Маршрутизация, Входящие порты, Внешний вид, Логи).
- **[`ProfileSelectActivity.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/ui/ProfileSelectActivity.kt)**:
  - Выбор и сортировка профилей, массовые операции (тест задержки, экспорт, удаление).
- **[`GroupSettingsActivity.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/ui/GroupSettingsActivity.kt)**:
  - Редактирование группы/подписки (URL, интервал автообновления, User-Agent, спуфинг приложений).
- **[`AdblockSettingsActivity.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/ui/AdblockSettingsActivity.kt)**:
  - Управление блокировщиком рекламы AdBlock (списки фильтров, HTTPS-фильтрация с установкой пользовательского CA сертификата, CNAME uncloaking).
- **[`LogcatActivity.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/ui/LogcatActivity.kt)**:
  - Интерактивный просмотрщик логов ядра в реальном времени с подсветкой синтаксиса, поиском и экспортом.
- **[`AppManagerActivity.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/ui/AppManagerActivity.kt)**:
  - Настройка раздельного туннелирования (выбор приложений, которые идут через прокси или в обход VPN).

---

### 2.11. Вспомогательные утилиты (`io.nekohasekai.sagernet.utils.*`)

- **[`PackageCache.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/utils/PackageCache.kt)**:
  - Асинхронное кэширование списка установленных Android-приложений (UID, имена пакетов, иконки) для раздельного туннелирования.
- **[`AdblockRepository.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/utils/AdblockRepository.kt)**:
  - Менеджер загрузки, кэширования и обновления списков правил AdBlock (uBlock, AdGuard форматы).
- **[`Theme.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/utils/Theme.kt)** и **[`CustomTheme.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/utils/CustomTheme.kt)**:
  - Система кастомизации UI: Material You Dynamic Colors, кастомные акцентные цвета, темная/светлая/ночная темы.
- **[`DefaultNetworkListener.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/io/nekohasekai/sagernet/utils/DefaultNetworkListener.kt)**:
  - Отслеживание смены физической сети (Wi-Fi <-> Cellular), смена DNS и IP-адресов интерфейса.

---

### 2.12. Модуль интеграции Matsuri NB4A (`moe.matsuri.nb4a.*`)

- **[`NativeInterface.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/moe/matsuri/nb4a/NativeInterface.kt)**:
  - JNI-мост для вызовов из Go-ядра `libcore` в Android runtime:
    - Защита сокетов `protect(fd)` от перехвата собственным VPN-интерфейсом.
    - Синхронизация статуса сети и адресов интерфейсов.
    - Обработка системных уведомлений и аутентификации.
- **[`LocalResolverImpl.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/moe/matsuri/nb4a/net/LocalResolverImpl.kt)**:
  - Реализация интерфейса локального резолвинга DNS Android для использования внутри Go `libcore`.
- **[`HevTunNative.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/moe/matsuri/nb4a/hevtun/HevTunNative.kt)** и **[`HevTunRuntime.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/moe/matsuri/nb4a/hevtun/HevTunRuntime.kt)**:
  - Запуск и управление встроенным стеком `hev-socks5-tunnel` (высокопроизводительный C-бинарник/библиотека для перенаправления TCP/UDP через локальный SOCKS5).
- **Кастомные протоколы Matsuri:**
  - `proxy/byedpi/`: Интеграция протокола обхода DPI ByeDPI.
  - `proxy/shadowtls/`: Интеграция протокола маскировки TLS ShadowTLS.
  - `proxy/anytls/`: Интеграция протокола AnyTLS.
- **[`HwidGenerator.kt`](file:///c:/Users/margo/OneDrive/Рабочий%20стол/cool/NekoBoxForAndroid/app/src/main/java/moe/matsuri/nb4a/utils/HwidGenerator.kt)**:
  - Генератор уникального идентификатора оборудования (Hardware ID) для защищенной авторизации при обновлении приватных подписок.




