import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { ChannelsPage } from "@/components/channels/ChannelsPage";
import { EventsPage } from "@/components/events/EventsPage";
import { RulesPage } from "@/components/rules/RulesPage";
import { HistoryPage } from "@/components/history/HistoryPage";
import { SettingsPage } from "@/components/settings/SettingsPage";
import {
  Bell,
  Radio,
  Filter,
  History,
  Settings,
} from "lucide-react";

function App() {
  const { t } = useTranslation();
  const [activeTab, setActiveTab] = useState("channels");

  return (
    <div className="h-screen flex flex-col">
      <div className="flex items-center justify-between px-4 pt-3 pb-1">
        <h1 className="text-lg font-semibold">{t("app.title")}</h1>
      </div>
      <Tabs
        value={activeTab}
        onValueChange={setActiveTab}
        className="flex-1 flex flex-col px-4 pb-4"
      >
        <TabsList className="grid w-full grid-cols-5">
          <TabsTrigger value="channels" className="flex items-center gap-1.5">
            <Radio className="h-4 w-4" />
            {t("tabs.channels")}
          </TabsTrigger>
          <TabsTrigger value="events" className="flex items-center gap-1.5">
            <Bell className="h-4 w-4" />
            {t("tabs.events")}
          </TabsTrigger>
          <TabsTrigger value="rules" className="flex items-center gap-1.5">
            <Filter className="h-4 w-4" />
            {t("tabs.rules")}
          </TabsTrigger>
          <TabsTrigger value="history" className="flex items-center gap-1.5">
            <History className="h-4 w-4" />
            {t("tabs.history")}
          </TabsTrigger>
          <TabsTrigger value="settings" className="flex items-center gap-1.5">
            <Settings className="h-4 w-4" />
            {t("tabs.settings")}
          </TabsTrigger>
        </TabsList>
        <TabsContent value="channels" className="flex-1 mt-4">
          <ChannelsPage />
        </TabsContent>
        <TabsContent value="events" className="flex-1 mt-4">
          <EventsPage />
        </TabsContent>
        <TabsContent value="rules" className="flex-1 mt-4">
          <RulesPage />
        </TabsContent>
        <TabsContent value="history" className="flex-1 mt-4">
          <HistoryPage />
        </TabsContent>
        <TabsContent value="settings" className="flex-1 mt-4">
          <SettingsPage />
        </TabsContent>
      </Tabs>
    </div>
  );
}

export default App;
