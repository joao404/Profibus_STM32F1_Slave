#include "WebService.h"

WebService *WebService::m_instance{nullptr};

WebService::WebService()
    : m_AutoConnect(m_WebServer),
      m_auxZ60Config("/", "Z60 Config"),
      m_progActive("progActive", "progActive", "Trackprogramming activ", false),
      m_readingLoco("readingLoco", "readingLoco", "Read locos from Mobile Station", false),
      m_saveButton("saveButton", "Save", "/z60configured")
{
    AutoConnectAux auxZ60Config("/", "Z60 Config");
    ACCheckbox(m_progActive, "progActive", "Trackprogramming activ", false);
    ACCheckbox(m_readingLoco, "readingLoco", "Read locos from Mobile Station", false);
    ACSubmit(m_saveButton, "Save", "/z60configured");

    m_WebServer.on("/can", [this]()
                   {
                        Serial.println("Can requested");
                        m_WebServer.send(200, "", ""); });

    m_WebServer.on("/config/prefs.cs2", [this]()
                   {
      Serial.println("prefs requested");
    m_WebServer.send(200, "text/plain", 
    F(
      "[Preferences]\nversion\n .major=0\n .minor=1\npage\n .entry\n ..key=Version\n ..value=\n"
      "page\n .entry\n ..key=SerNum\n ..value=84\n .entry\n ..key=GfpUid\n ..value=1129525928\n .entry\n ..key=GuiUid\n"
      " ..value=1129525929\n .entry\n ..key=HardVers\n ..value=3.1\n"
    )); });

    m_WebServer.on("/config/magnetartikel.cs2", [this]()
                   {
      Serial.println("magnetartikel requested");
    m_WebServer.send(200, "text/plain",
    F(
      "[magnetartikel]\n"
      "version\n"
      " .minor=1\n"
    )); });

    m_WebServer.on("/config/gleisbild.cs2", [this]()
                   {
      Serial.println("gleisbild requested");
    m_WebServer.send(200, "text/plain",
    F(
      "[gleisbild]\n"
      "version\n"
      " .major=1\n"
      "groesse\n"
      "zuletztBenutzt\n"
      " .name=gleisbildDummy\n"
      "seite\n"
      " .name=gleisbildDummy\n"
    )); });

    m_WebServer.on("/config/fahrstrassen.cs2", [this]()
                   {
      Serial.println("fahrstrassen requested");
    m_WebServer.send(200, "text/plain",
    F(
      "[fahrstrassen]\n"
      "version\n"
      " .minor=4\n"
    )); });

    m_WebServer.on("/config/gleisbilder/gleisbildDummy.cs2", [this]()
                   {
      Serial.println("gleisbildDummy requested");
    m_WebServer.send(200, "text/plain",
    F(
      "[gleisbildseite]\n"
      "version\n"
      " .major=1\n"
    )); });

    m_WebServer.on("/config/geraet.vrs", [this]()
                   {
      Serial.println("geraet requested");
    m_WebServer.send(200, "text/plain",
    F(
"[geraet]\n"
"version\n"
" .minor=1\n"
"geraet\n"
" .sernum=1\n"
" .hardvers=ESP,1\n"
""
    )); });
}

WebService *WebService::getInstance()
{
    if (nullptr == m_instance)
    {
        m_instance = new WebService();
    }
    return m_instance;
};

WebService::~WebService()
{
}

void WebService::cyclic()
{
    m_AutoConnect.handleClient();
}

void WebService::begin(AutoConnectConfig &autoConnectConfig, void (*programmingFkt)(bool), void (*readingFkt)(void))
{
    m_programmingFkt = programmingFkt;

    m_readingFkt = readingFkt;

    m_WebServer.on("/z60configured", [this]()
                   {
    if (m_WebServer.hasArg("progActive"))
    {
      Serial.println("setProgramming(true)");
      m_programmingFkt(true);
    }
    else
    {
      Serial.println("setProgramming(false)");
      m_programmingFkt(false);
    }
    if (m_WebServer.hasArg("readingLoco"))
    {
      Serial.println("trigger loco reading");
      m_readingFkt();
    }
    m_WebServer.send(200, "text/plain", "Ongoing"); });

    m_AutoConnect.config(autoConnectConfig);

    m_AutoConnect.onNotFound(WebService::handleNotFound);

    m_auxZ60Config.add({m_progActive, m_readingLoco, m_saveButton});

    m_AutoConnect.join(m_auxZ60Config);

    m_AutoConnect.begin();
}

void WebService::setLokomotiveAvailable(bool isAvailable)
{
    m_lokomotiveAvailable = isAvailable;
}

void WebService::handleNotFound(void)
{
    const String filePath = m_instance->m_WebServer.uri();
    Serial.print(filePath);
    Serial.println(" requested");

    if (m_instance->getContentType(filePath) == "image/png")
    {
        Serial.print(m_instance->m_WebServer.uri());
        Serial.println(" requested");
        if (filePath.startsWith("/images/cs2/fcticons"))
        {
            String requestedFile = filePath.substring(11);
            if (SPIFFS.exists(requestedFile.c_str()))
            {
                File uploadedFile = SPIFFS.open(requestedFile.c_str(), "r");
                String mime = m_instance->getContentType(requestedFile);
                m_instance->m_WebServer.streamFile(uploadedFile, mime);
                uploadedFile.close();
            }
            else
            {
                m_instance->m_WebServer.send(404, "text/plain", "png not available");
            }
        }
        else if (SPIFFS.exists("/default.png"))
        {
            Serial.println("Send default.png");
            File uploadedFile = SPIFFS.open("/default.png", "r");
            m_instance->m_WebServer.streamFile(uploadedFile, "image/png");
            uploadedFile.close();
        }
        else
        {
            m_instance->m_WebServer.send(404, "text/plain", "png not available");
        }
    }
    else if (SPIFFS.exists(filePath.c_str()))
    {
        if (strcmp("/config/lokomotive.cs2", filePath.c_str()) == 0)
        {
            if (!m_instance->m_lokomotiveAvailable)
            {
                m_instance->m_WebServer.send(404, "text/plain", "lokomotive.cs2 under construction");
                return;
            }
        }
        File uploadedFile = SPIFFS.open(filePath.c_str(), "r");
        String mime = m_instance->getContentType(filePath);
        m_instance->m_WebServer.streamFile(uploadedFile, mime);
        uploadedFile.close();
    }
    else
    {
        String message = "File Not Found\n";
        message += "URI: ";
        message += m_instance->m_WebServer.uri();
        message += "\nMethod: ";
        message += (m_instance->m_WebServer.method() == HTTP_GET) ? "GET" : "POST";
        message += "\nArguments: ";
        message += m_instance->m_WebServer.args();
        message += "\n";
        for (uint8_t i = 0; i < m_instance->m_WebServer.args(); i++)
        {
            message += " " + m_instance->m_WebServer.argName(i) + ": " + m_instance->m_WebServer.arg(i) + "\n";
        }
        Serial.print(message);
        m_instance->m_WebServer.send(404, "text/plain", message);
    }
}

String WebService::getContentType(const String &filename)
{
    if (filename.endsWith(".cs2"))
    {
        return "text/plain";
    }
    else if (filename.endsWith(".txt"))
    {
        return "text/plain";
    }
    else if (filename.endsWith(".htm"))
    {
        return "text/html";
    }
    else if (filename.endsWith(".html"))
    {
        return "text/html";
    }
    else if (filename.endsWith(".css"))
    {
        return "text/css";
    }
    else if (filename.endsWith(".js"))
    {
        return "application/javascript";
    }
    else if (filename.endsWith(".json"))
    {
        return "application/json";
    }
    else if (filename.endsWith(".png"))
    {
        return "image/png";
    }
    else if (filename.endsWith(".gif"))
    {
        return "image/gif";
    }
    else if (filename.endsWith(".jpg"))
    {
        return "image/jpeg";
    }
    else if (filename.endsWith(".jpeg"))
    {
        return "image/jpeg";
    }
    else if (filename.endsWith(".ico"))
    {
        return "image/x-icon";
    }
    else if (filename.endsWith(".svg"))
    {
        return "image/svg+xml";
    }
    else if (filename.endsWith(".xml"))
    {
        return "text/xml";
    }
    else if (filename.endsWith(".pdf"))
    {
        return "application/x-pdf";
    }
    else if (filename.endsWith(".zip"))
    {
        return "application/x-zip";
    }
    else if (filename.endsWith(".gz"))
    {
        return "application/x-gzip";
    }
    else if (filename.endsWith(".z21"))
    {
        return "application/octet-stream";
    }
    return "text/plain";
}