import requests
from urllib.parse import quote
from bs4 import BeautifulSoup

# Proof of concept login for Fourth API

USER = "username@email.com"
PASSWORD = "Password123"

s = requests.Session()

ORDER_BY = "StartDateTime+asc"
FROM = quote("2023/01/18", safe='')
TO = quote("2023/01/21", safe='')
API = f"https://api.fourth.com/api/myschedules/schedule?%24orderby={ORDER_BY}&%24top=50&fromDate={FROM}&toDate={TO}"

### Phase 0 ### Get ViewState & Other stuff

print("Phase 0")
BaseLogin = s.get(API, allow_redirects=True)
BaseLoginSoup = BeautifulSoup(BaseLogin.text, 'html.parser')
oinfo_url = BaseLoginSoup.find_all('form')[0].attrs['action']
view_state = BaseLoginSoup.find_all('input', id="com.salesforce.visualforce.ViewState")[0].attrs['value']
view_state_version = BaseLoginSoup.find_all('input', id="com.salesforce.visualforce.ViewStateVersion")[0].attrs['value']
view_state_mac = BaseLoginSoup.find_all('input', id="com.salesforce.visualforce.ViewStateMAC")[0].attrs['value']

### PHASE 1 FMPLogin ### uname pword -> oinfo
print("Phase 1")
FORM = {
    "j_id0:j_id2:j_id15": "j_id0:j_id2:j_id15",
    "j_id0:j_id2:j_id15:username": USER,
    "j_id0:j_id2:j_id15:j_id24": PASSWORD,
    "j_id0:j_id2:j_id15:submit": "Sign+In",
    "com.salesforce.visualforce.ViewState": view_state,
    "com.salesforce.visualforce.ViewStateVersion": view_state_version,
    "com.salesforce.visualforce.ViewStateMAC": view_state_mac,
}

FMPLogin = s.post(oinfo_url, data=FORM)
FMPLoginSoup = BeautifulSoup(FMPLogin.text, 'html.parser')
frontdoor_url = FMPLoginSoup.find_all('script')[0].string.split("window.location.href =\'")[1].split("\';")[0]
### PHASE 2 FRONT DOOR ### oinfo -> clientSrc, inst, lloopch_loid, lloopch_lpid, oid, sid, sid_Client
# get front door URL from FMPLogin response body, window.location.href
print("Phase 2")

frontdoor = s.get(frontdoor_url)
frontdoor_soup = BeautifulSoup(frontdoor.text, 'html.parser')
idp_login_url = "https://secure.fourth.com" + frontdoor_soup.find_all('a')[0].attrs["href"]

# Gives clientSrc, inst, lloopch_loid, lloopch_lpid, oid, sid, sid_Client
# FrontDoor = s.get(FRONT_DOOR)
# print(FrontDoor.status_code)

### PHASE 3 LOGIN ### clientSrc, inst, oid, oinfo, sid, sid_Client
print("Phase 3")

idp_login = s.get(idp_login_url)
idp_login_soup = BeautifulSoup(idp_login.text, 'html.parser')
saml_url = idp_login_soup.find_all('form')[0].attrs['action']
idp_relay_state = idp_login_soup.find_all('input')[0].attrs["value"]
idp_saml_response = idp_login_soup.find_all('input')[1].attrs["value"]

# Gives RelayState and SAMLResponse in body of response along with URL for GenerateTokenFromSAML all as form stuff

### PHASE 4 GenerateTokenFromSAML ###
print("Phase 4")

saml_form = {"RelayState": idp_relay_state, "SAMLResponse": idp_saml_response}
saml = s.post(saml_url, data=saml_form)

print("Done!")
print(f"l7auth_prod: {s.cookies['l7auth_prod']}")

