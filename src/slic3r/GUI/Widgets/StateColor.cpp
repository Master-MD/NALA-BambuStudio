#include "StateColor.hpp"
#include <wx/gdicmn.h>
#include <cstdlib>
#include <fstream>
#include <string>

static bool gDarkMode = false;
static std::string gAppliedNalaSkin;

static std::string nala_resolve_active_skin()
{
    if (const char* env_skin = std::getenv("NALA_SKIN"); env_skin && *env_skin)
        return std::string(env_skin);
    if (const char* home = std::getenv("HOME"); home && *home) {
        std::ifstream ifs(std::string(home) + "/.config/NALA-BambuStudio/active_skin");
        std::string line;
        if (ifs && std::getline(ifs, line)) {
            while (!line.empty() && (line.back() == '\r' || line.back() == '\n' || line.back() == ' '))
                line.pop_back();
            if (!line.empty())
                return line;
        }
    }
    return "nala-obsidian";
}

static bool operator<(wxColour const &l, wxColour const &r) { return l.GetRGBA() < r.GetRGBA(); }

// Light → dark color mapping
static std::map<wxColour, wxColour> gDarkColors{
    {ThemeColor::BrandGreen,  "#21A452"},/*green*/
    {ThemeColor::BrandGreenPressed, "#1C8A46"},
    {ThemeColor::BrandGreenHovered, "#37B865"},
    // {"#1F8EEA", "#2778D2"},/*blue*/ -- dead, only used by disabled Notebook.cpp:80 OnPaint
    {ThemeColor::Warning,     "#D15B00"},
    {ThemeColor::Danger,      "#BB2A3A"},/*red*/
    {ThemeColor::Link,        "#479EF5"},/*blue*/
    {ThemeColor::TextPrimary, "#EFEFF0"},/*black*/
    {"#2C2C2E", "#B3B3B4"},/*black*/
    {"#E5E7EB", "#374151"},/*gray200 -> gray800*/
    {ThemeColor::TextMuted,   "#818183"},/*gray -> */    // same hex as Grey700
    {ThemeColor::TextMutedBright, "#B7B7BB"}, // #fff @ 60% opacity over Grey300's dark mapping
    {"#ACACAC", "#54545A"},/*gray -> */
    {ThemeColor::Grey300,     "#4C4C55"},/*gray -> */
    {ThemeColor::Grey350,     "#3E3E45"},
    {ThemeColor::TextSecondary, "#E5E5E4"}, // #323A3D
    {ThemeColor::TextDisabled,  "#5A5A5A"}, // #909090
    {ThemeColor::White,       "#2D2D31"},
    {ThemeColor::Grey200,     "#36363C"},
    {ThemeColor::Grey250,     "#36363B"},
    {"#3B4446", "#2D2D30"},
    {ThemeColor::Grey400,     "#54545B"},
    {"#DBFDD5", "#3B3B40"},
    {"#000000", "#FFFFFE"},
    {"#F4F4F4", "#36363D"},
    {"#F7F7F7", "#333337"},
    {"#DBDBDB", "#4A4A51"},
    {ThemeColor::LightGreen,  "#283232"},
    {"#323A3C", "#E5E5E6"},
    {"#6B6B6A", "#B3B3B5"},
    {"#303A3C", "#E5E5E5"},
    {"#FEFFFF", "#242428"},
    {ThemeColor::Grey450,     "#2D2D29"}, // #A6A9AA
    {"#363636", "#B2B3B5"},
    {"#F0F0F1", "#404040"},
    {ThemeColor::Grey500,     "#53545A"},
    {"#D7E8DE", "#1F2B27"},
    {"#2B3436", "#808080"},
    {"#ABABAB", "#ABABAB"},
    {"#D9D9D9", "#2D2D32"},
    {"#EBF9F0", "#293F34"},
    {"#DBFDE7", "#1F3529"},
    {"#ADADAD", "#D2D2D2"}
    //{"#F0F0F0", "#4C4C54"},
};

std::map<wxColour, wxColour> const & StateColor::GetDarkMap()
{
    return gDarkColors;
}

static void nala_apply_skin_palette()
{
    const std::string skin = nala_resolve_active_skin();
    if (skin == gAppliedNalaSkin)
        return;
    gAppliedNalaSkin = skin;

    if (skin == "bambu-classic") {
        gDarkColors[ThemeColor::BrandGreen]        = wxColour("#21A452");
        gDarkColors[ThemeColor::BrandGreenPressed] = wxColour("#1C8A46");
        gDarkColors[ThemeColor::BrandGreenHovered] = wxColour("#37B865");
        gDarkColors[ThemeColor::White]             = wxColour("#2D2D31");
        gDarkColors[ThemeColor::Grey200]           = wxColour("#36363C");
        gDarkColors[ThemeColor::Grey250]           = wxColour("#36363B");
        return;
    }

    if (skin == "porsche-v4") {
        gDarkColors[ThemeColor::BrandGreen]        = wxColour("#D5001C");
        gDarkColors[ThemeColor::BrandGreenPressed] = wxColour("#A80016");
        gDarkColors[ThemeColor::BrandGreenHovered] = wxColour("#F21B37");
        gDarkColors[ThemeColor::White]             = wxColour("#17181E");
        gDarkColors[ThemeColor::Grey200]           = wxColour("#20222A");
        gDarkColors[ThemeColor::Grey250]           = wxColour("#0E0E12");
        gDarkColors[ThemeColor::LightGreen]        = wxColour("#2A1519");
        return;
    }

    if (skin == "doom-uac") {
        gDarkColors[ThemeColor::BrandGreen]        = wxColour("#EA580C");
        gDarkColors[ThemeColor::BrandGreenPressed] = wxColour("#C2410C");
        gDarkColors[ThemeColor::BrandGreenHovered] = wxColour("#F97316");
        gDarkColors[ThemeColor::White]             = wxColour("#181311");
        gDarkColors[ThemeColor::Grey200]           = wxColour("#231B18");
        gDarkColors[ThemeColor::Grey250]           = wxColour("#0C0A09");
        gDarkColors[ThemeColor::LightGreen]        = wxColour("#2B1910");
        return;
    }

    if (skin == "matrix-construct") {
        gDarkColors[ThemeColor::BrandGreen]        = wxColour("#10B981");
        gDarkColors[ThemeColor::BrandGreenPressed] = wxColour("#047857");
        gDarkColors[ThemeColor::BrandGreenHovered] = wxColour("#34D399");
        gDarkColors[ThemeColor::White]             = wxColour("#07130E");
        gDarkColors[ThemeColor::Grey200]           = wxColour("#0B1D15");
        gDarkColors[ThemeColor::Grey250]           = wxColour("#030806");
        gDarkColors[ThemeColor::LightGreen]        = wxColour("#0A261A");
        return;
    }

    // Default: nala-obsidian
    gDarkColors[ThemeColor::BrandGreen]        = wxColour("#10B981");
    gDarkColors[ThemeColor::BrandGreenPressed] = wxColour("#059669");
    gDarkColors[ThemeColor::BrandGreenHovered] = wxColour("#34D399");
    gDarkColors[ThemeColor::White]             = wxColour("#11151E");
    gDarkColors[ThemeColor::Grey200]           = wxColour("#171C28");
    gDarkColors[ThemeColor::Grey250]           = wxColour("#080A0F");
    gDarkColors[ThemeColor::LightGreen]        = wxColour("#0F241D");
}

void StateColor::SetDarkMode(bool dark) {
    gDarkMode = dark;
    nala_apply_skin_palette();
}

inline wxColour darkModeColorFor2(wxColour const &color)
{
    if (!gDarkMode)
        return color;
    if (gAppliedNalaSkin.empty())
        nala_apply_skin_palette();
    auto iter = gDarkColors.find(color);
    if (iter != gDarkColors.end()) return iter->second;
    return color;
}

std::map<wxColour, wxColour> revert(std::map<wxColour, wxColour> const & map)
{
    std::map<wxColour, wxColour> map2;
    for (auto &p : map) map2.emplace(p.second, p.first);
    return map2;
}

wxColour StateColor::lightModeColorFor(wxColour const &color)
{
    static std::map<wxColour, wxColour> gLightColors = revert(gDarkColors);
    auto iter = gLightColors.find(color);
    if (iter != gLightColors.end()) return iter->second;
    return color;
}

wxColour StateColor::darkModeColorFor(wxColour const &color) { return darkModeColorFor2(color); }

StateColor::StateColor(wxColour const &color) { append(color, 0); }

StateColor::StateColor(wxString const &color) { append(color, 0); }

StateColor::StateColor(unsigned long color) { append(color, 0); }

void StateColor::append(wxColour const & color, int states)
{
    statesList_.push_back(states);
    colors_.push_back(color);
}

void StateColor::append(wxString const & color, int states)
{
    wxColour c1(color);
    append(c1, states);
}

void StateColor::append(unsigned long color, int states)
{
    if ((color & 0xff000000) == 0)
        color |= 0xff000000;
    wxColour cl; cl.SetRGBA(color & 0xff00ff00 | ((color & 0xff) << 16) | ((color >> 16) & 0xff));
    append(cl, states);
}

void StateColor::clear()
{
    statesList_.clear();
    colors_.clear();
}

int StateColor::states() const
{
    int states = 0;
    for (auto s : statesList_) states |= s;
    states = (states & 0xffff) | (states >> 16);
    if (takeFocusedAsHovered_ && (states & Hovered))
        states |= Focused;
    return states;
}

wxColour StateColor::defaultColor() {
    return colorForStates(0);
}

wxColour StateColor::colorForStates(int states)
{
    bool focused = takeFocusedAsHovered_ && (states & Focused);
    for (int i = 0; i < statesList_.size(); ++i) {
        int s = statesList_[i];
        int on = s & 0xffff;
        int off = s >> 16;
        if ((on & states) == on && (off & ~states) == off) {
            return darkModeColorFor2(colors_[i]);
        }
        if (focused && (on & Hovered)) {
            on |= Focused;
            on &= ~Hovered;
            if ((on & states) == on && (off & ~states) == off) {
                return darkModeColorFor2(colors_[i]);
            }
        }
    }
    return wxColour(0, 0, 0, 0);
}

wxColour StateColor::colorForStatesNoDark(int states)
{
    bool focused = takeFocusedAsHovered_ && (states & Focused);
    for (int i = 0; i < statesList_.size(); ++i) {
        int s = statesList_[i];
        int on = s & 0xffff;
        int off = s >> 16;
        if ((on & states) == on && (off & ~states) == off) {
            return colors_[i];
        }
        if (focused && (on & Hovered)) {
            on |= Focused;
            on &= ~Hovered;
            if ((on & states) == on && (off & ~states) == off) {
                return colors_[i];
            }
        }
    }
    return wxColour(0, 0, 0, 0);
}

int StateColor::colorIndexForStates(int states)
{
    for (int i = 0; i < statesList_.size(); ++i) {
        int s   = statesList_[i];
        int on  = s & 0xffff;
        int off = s >> 16;
        if ((on & states) == on && (off & ~states) == off) { return i; }
    }
    return -1;
}

bool StateColor::setColorForStates(wxColour const &color, int states)
{
    for (int i = 0; i < statesList_.size(); ++i) {
        if (statesList_[i] == states) {
            colors_[i] = color;
            return true;
        }
    }
    return false;
}

void StateColor::setTakeFocusedAsHovered(bool set) { takeFocusedAsHovered_ = set; }

StateColor StateColor::createButtonStyleGray()
{
    return StateColor(std::pair<wxColour, int>(wxColour(206, 206, 206), StateColor::Pressed),
        std::pair<wxColour, int>(*wxWHITE, StateColor::Focused),
        std::pair<wxColour, int>(wxColour(238, 238, 238), StateColor::Hovered),
        std::pair<wxColour, int>(*wxWHITE, StateColor::Normal));
}
