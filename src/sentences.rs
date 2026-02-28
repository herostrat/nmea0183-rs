//! All the supported sentence type data and parsers.

pub mod aam;
pub mod alm;
pub mod apa;
pub mod apb;
pub mod bod;
pub mod bwc;
pub mod bww;
pub mod dbk;
pub mod dbs;
pub mod dbt;
pub mod dpt;
pub mod dsc;
pub mod gbs;
pub mod gga;
pub mod gll;
pub mod gns;
pub mod gsa;
pub mod gst;
pub mod gsv;
pub mod hdg;
pub mod hdm;
pub mod hdt;
pub mod hsc;
pub mod mda;
pub mod mtw;
pub mod mwd;
pub mod mwv;
pub mod rmb;
pub mod rmc;
pub mod rmz;
pub mod rot;
pub mod rpm;
pub mod rsa;
pub mod stalk;
pub mod ttm;
pub mod txt;
pub mod utils;
pub mod vdm;
pub mod vdr;
pub mod vhw;
pub mod vlw;
pub mod vpw;
pub mod vtg;
pub mod vwr;
pub mod vwt;
pub mod xdr;
pub mod xte;
pub mod wnc;
pub mod zda;
pub mod zfo;
pub mod ztg;

pub mod faa_mode;
pub mod fix_type;
pub mod gen_utils;
pub mod gnss_type;

#[doc(inline)]
pub use {
    aam::{AamData, parse_aam},
    alm::{AlmData, parse_alm},
    apa::{ApaData, parse_apa},
    apb::{ApbData, parse_apb},
    bod::{BodData, parse_bod},
    bwc::{BwcData, parse_bwc},
    bww::{BwwData, parse_bww},
    dbk::{DbkData, parse_dbk},
    dbs::{DbsData, parse_dbs},
    dbt::{DbtData, parse_dbt},
    dpt::{DptData, parse_dpt},
    dsc::{DscData, parse_dsc},
    faa_mode::{FaaMode, FaaModes},
    fix_type::FixType,
    gbs::{GbsData, parse_gbs},
    gga::{GgaData, parse_gga, parse_ggae},
    gll::{GllData, parse_gll},
    gns::{GnsData, parse_gns},
    gnss_type::GnssType,
    gsa::{GsaData, parse_gsa},
    gst::{GstData, parse_gst},
    gsv::{GsvData, parse_gsv},
    hdg::{HdgData, parse_hdg},
    hdm::{HdmData, parse_hdm},
    hdt::{HdtData, parse_hdt},
    hsc::{HscData, parse_hsc},
    mda::{MdaData, parse_mda},
    mtw::{MtwData, parse_mtw},
    mwd::{MwdData, parse_mwd},
    mwv::{MwvData, parse_mwv},
    rmb::{RmbData, parse_rmb},
    rmc::{RmcData, parse_rmc, parse_rmce},
    rmz::{PgrmzData, parse_pgrmz},
    rot::{RotData, parse_rot},
    rpm::{RpmData, RpmSource, parse_rpm},
    rsa::{RsaData, parse_rsa},
    stalk::{StalkData, parse_stalk},
    ttm::{
        TtmAngle, TtmData, TtmDistanceUnit, TtmReference, TtmStatus, TtmTypeOfAcquisition,
        parse_ttm,
    },
    txt::{TxtData, parse_txt},
    vdm::{VdmData, parse_vdm},
    vdr::{VdrData, parse_vdr},
    vhw::{VhwData, parse_vhw},
    vlw::{VlwData, parse_vlw},
    vpw::{VpwData, parse_vpw},
    vtg::{VtgData, parse_vtg},
    vwr::{VwrData, parse_vwr},
    vwt::{VwtData, parse_vwt},
    xdr::{XdrData, XdrMeasurement, parse_xdr},
    xte::{XteData, parse_xte},
    wnc::{WncData, parse_wnc},
    zda::{ZdaData, parse_zda},
    zfo::{ZfoData, parse_zfo},
    ztg::{ZtgData, parse_ztg},
};

pub(crate) fn nom_parse_failure(inp: &str) -> nom::Err<nom::error::Error<&str>> {
    nom::Err::Failure(nom::error::Error::new(inp, nom::error::ErrorKind::Fail))
}
