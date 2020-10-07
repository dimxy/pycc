use num_bigint::{BigInt, BigUint};
use secp256k1::{PublicKey, Signature, SecretKey, Message, sign};


/*use num_traits::cast::FromPrimitive;
use simple_asn1::{to_der, ASN1Block, ASN1Class};
use std::collections::HashSet;*/

//use rusthex::ToHex;
use base64::*;
//extern crate hex;
use hex::*;

//use base64::Base64Mode;
//mod jscc {

use crate::Condition::*; 
use crate::*; 

use log::Level;
use log::info;

use js_sys::Reflect::*;
use js_sys::Uint8ClampedArray;
use js_sys::Uint8Array;
use wasm_bindgen::JsCast;

//use wasm_bindgen::Clamped;

use wasm_bindgen::prelude::*;

//#[wasm_bindgen]
//extern {
//    pub fn JsCConditionBinary(jscond: &JsValue) -> Result<JsValue, JsValue>;
//}

fn cast_js_value_to_u16(js_val: &JsValue) -> u16
{
    if js_val.is_string()   {
        let r: u16 = js_val.as_string().unwrap().parse().unwrap();
        r   
    }
    else {
        let r: f64 = js_val.as_f64().unwrap();
        r as u16
    }
}

fn parse_js_cond(js_cond: &JsValue) -> Result<Condition, JsValue> 
{
    let js_type = js_sys::Reflect::get(&js_cond, &JsValue::from_str("type"))?;
    if !js_type.is_string() {
        return Err("no \'type\" property".into());
    }
    match js_type.as_string().unwrap().as_ref() {
        "threshold-sha-256" => {
            let js_threshold = js_sys::Reflect::get(&js_cond, &JsValue::from_str("threshold"))?;
            if js_threshold.is_null()  {
                return Err("no \'threshold\" property".into());
            }

            //let t = js_threshold.as_f64().unwrap();
            //info!("threshold step 0.1 value {}", t);

            let js_subfulfillments = js_sys::Reflect::get(&js_cond, &JsValue::from_str("subfulfillments"))?;
            if !js_sys::Array::is_array(&js_subfulfillments) {
                return Err("no \'subfulfillments\" array".into());
            }
            info!("threshold step 1");
            
            let array = js_sys::Array::from(&js_subfulfillments);
            info!("threshold step 2 array.size={}", array.length().to_string());

            let mut subconds = vec![];
            /*array.for_each(&mut |elem, _, _| {
                //if let Condition::Threshold{threshold: _, subconditions } = &mut cond {
                //    subconditions.push(parse_js_cond(&elem).unwrap());
                //}
                info!("threshold step 2.1");

                let subcond = parse_js_cond(&elem)?;
                info!("threshold step 2.2");

                subconds.push(subcond);
            });*/

            for elem in array.iter() {
                info!("threshold step 2.1");

                let subcond = parse_js_cond(&elem)?; // propagate Err up
                info!("threshold step 2.2");

                subconds.push(subcond);
            }

            info!("threshold step 3");
            info!("threshold step 3 js_threshold is str={} obj={} null={} function={} falsy={} symbol={}", js_threshold.is_string(), js_threshold.is_object(), js_threshold.is_null(), js_threshold.is_function(), js_threshold.is_falsy(), js_threshold.is_symbol());
            //info!("threshold step 3 str={}", js_threshold.as_string().unwrap());

            /*let threshold_val = match js_threshold.dyn_into::<bool>() {
                Ok(val) => val,
                Err(e) => return Err(JsValue::from_str("could no parse threshold")), 
            };*/
            let threshold_val = cast_js_value_to_u16(&js_threshold);
            info!("threshold step 3.1 value {}", threshold_val);

            let cond = Threshold {
                threshold: threshold_val as u16,
                subconditions: subconds
            };
            info!("threshold step 4 threshold_val={}", threshold_val);

            Ok(cond)
        },
        "eval-sha-256" => {
            let js_code = js_sys::Reflect::get(&js_cond, &JsValue::from_str("code"))?;
            if js_code.is_null()  {
                return Err("no \'code\" property".into());
            }

            info!("eval step 0, code={}", js_code.as_string().unwrap());
            let code_decoded = base64::decode( js_code.as_string().unwrap() ).unwrap();
            info!("eval step 1, code={}", hex::encode( &code_decoded ));
            //info!("eval step 1, code={}", js_code.as_string().unwrap().parse::<u32>().unwrap());
            
            let cond = Eval {
                //code: js_code.as_string().unwrap().parse::<u32>().unwrap().to_le_bytes().iter().cloned().collect()
                code: code_decoded
            };
            info!("eval step 2");


            Ok(cond)
        },
        "secp256k1-sha-256" => {
            let js_publicKey = js_sys::Reflect::get(&js_cond, &JsValue::from_str("publicKey"))?;
            if js_publicKey.is_null()  {
                return Err("no \'publicKey\" property".into());
            }
            info!("secp step 1");
            let pk = hex::decode( js_publicKey.as_string().unwrap() ).unwrap();
                        
            let js_signature = js_sys::Reflect::get(&js_cond, &JsValue::from_str("signature"))?;
            let sig_value = match js_signature.is_string() {
                true => Some(Signature::parse_slice(&hex::decode( js_signature.as_string().unwrap() ).unwrap()).unwrap()),
                false => None,
            };

            let cond = Secp256k1 {
                pubkey: PublicKey::parse_slice(&pk, None).unwrap(),
                signature: sig_value
            };
            info!("secp step 2");

            Ok(cond)
        },
        _ => {
            return Err("unknown".into());
        }
    }

    //Ok(JsValue::from("hello"))
}

fn make_js_cond(cond: Condition) -> Result<JsValue, JsValue> 
{
    let js_cond = js_sys::Object::new();

    match cond {
        Secp256k1 { pubkey, signature } => {
            js_sys::Reflect::set(&js_cond, &JsValue::from_str("type"), &JsValue::from_str("secp256k1-sha-256"))?;
            js_sys::Reflect::set(&js_cond, &JsValue::from_str("publicKey"), &JsValue::from_str(&hex::encode( &pubkey.serialize_compressed() )))?;
            info!("checking signature...");
            if signature != None {
                info!("found signature!");
                js_sys::Reflect::set(&js_cond, &JsValue::from_str("signature"), &JsValue::from_str(&hex::encode( &signature.unwrap().serialize() )))?;
            }
        }
        Eval { code } => {
            js_sys::Reflect::set(&js_cond, &JsValue::from_str("type"), &JsValue::from_str("eval-sha-256"))?;
            js_sys::Reflect::set(&js_cond, &JsValue::from_str("code"), &JsValue::from_str(&base64::encode( code )))?;
        }
        Preimage { preimage } => {
            js_sys::Reflect::set(&js_cond, &JsValue::from_str("type"), &JsValue::from_str("preimage-sha-256"))?;  // TODO: unfinished...
        }
        Threshold {
            threshold,
            subconditions,
        } => {
            let js_subconds = js_sys::Array::new();
            for subcond in subconditions    { 
                let js_subcond = make_js_cond(subcond)?;
                js_subconds.push(&js_subcond);
            }
            js_sys::Reflect::set(&js_cond, &JsValue::from_str("type"), &JsValue::from_str("threshold-sha-256"))?;
            js_sys::Reflect::set(&js_cond, &JsValue::from_str("threshold"), &JsValue::from_str( &threshold.to_string() ))?;
            js_sys::Reflect::set(&js_cond, &JsValue::from_str("subfulfillments"), &js_subconds.as_ref())?;

        }
        Anon { ref fingerprint, ref cost, ref subtypes, .. } => {
            js_sys::Reflect::set(&js_cond, &JsValue::from_str("type"), &JsValue::from_str("(anon)"))?;
            js_sys::Reflect::set(&js_cond, &JsValue::from_str("fingerprint"), &JsValue::from_str(&base64::encode( fingerprint )))?;
            js_sys::Reflect::set(&js_cond, &JsValue::from_str("cost"), &JsValue::from_str( &cost.to_string() ))?;
            if  cond.get_type().has_subtypes()   {
                let vsubtypes = internal::pack_set(subtypes.clone());
                // convert vec[4] to u32
                info!("vsubtypes.len {}", vsubtypes.len());
                if vsubtypes.len() > 4 {
                    return Err(JsValue::from("Internal error: expected subtypes as Vec of 4"));
                }
                let mut asubtypes: [u8; 4] = [0,0,0,0];
                let mut i = 0;
                while i < vsubtypes.len()   {
                    asubtypes[i] = vsubtypes[i]; 
                    i += 1;
                }
                /*asubtypes[1] = vsubtypes[1]; 
                asubtypes[2] = vsubtypes[2]; 
                asubtypes[3] = vsubtypes[3]; */
                //let boxed_slice = vsubtypes.into_boxed_slice();
                //let boxed_array: Box<[u8; 4]> = match boxed_slice.try_into() {
                //    Ok(ba) => ba,
                //    Err(o) => Err("Internal error: expected subtypes a Vec of 4"),
                //};
                //js_sys::Reflect::set(&js_cond, &JsValue::from_str("subtypes"), &JsValue::from_str(  &u32::from_be_bytes(asubtypes).to_string()  ))?;
                js_sys::Reflect::set(&js_cond, &JsValue::from_str("subtypes"), &JsValue::from_str(  &u32::from_be_bytes(asubtypes).to_string()  ))?;

            }
        }
    }
    Ok(JsValue::from(js_cond))
}

#[wasm_bindgen]
pub fn js_cc_condition_binary(js_cond: &JsValue) -> Result<Uint8ClampedArray, JsValue> 
{
    console_log::init_with_level(Level::Debug);
    
    let cond = parse_js_cond(js_cond)?;
    let encoded_cond = cond.encode_condition(); // no error returned

    //Ok(JsValue::from_str(&String::from_utf8_lossy(&encoded_cond)))
    //Ok(JsValue::from_str(&hex::encode(&encoded_cond)))
    //Ok(JsValue::from_str(&hex::encode(&encoded_cond)))
    Ok(Uint8ClampedArray::from(encoded_cond.as_slice()))
    //let jsthreshold = js_sys::Reflect::get(&target, "Threshold")?;
}

#[wasm_bindgen]
pub fn js_cc_fulfillment_binary(js_cond: &JsValue) -> Result<Uint8ClampedArray, JsValue> 
{
    console_log::init_with_level(Level::Debug);
    
    let cond = match parse_js_cond(js_cond)  {
        Ok(c) => c,
        Err(e) => { info!("could not parse cond"); return Err(JsValue::from_str("could parse cc")) },
    };
    let encoded_ffil = cond.encode_fulfillment()?;

    //Ok(JsValue::from_str(&String::from_utf8_lossy(&encoded_cond)))
    //Ok(JsValue::from_str(&hex::encode(&encoded_cond)))
    //Ok(JsValue::from_str(&hex::encode(&encoded_cond)))
    Ok(Uint8ClampedArray::from(encoded_ffil.as_slice()))
    //Ok(Uint8ClampedArray::from(JsValue::from_str("00")))

    //let jsthreshold = js_sys::Reflect::get(&target, "Threshold")?;
}

#[wasm_bindgen]
pub fn js_sign_secp256k1(js_cond: &JsValue, uca_secret_key: &Uint8ClampedArray, uca_msg: &Uint8ClampedArray) -> Result<JsValue, JsValue> 
{
    console_log::init_with_level(Level::Debug);
    
    let mut cond = parse_js_cond(js_cond)?;
    let secret_key = SecretKey::parse_slice(&uca_secret_key.to_vec()).unwrap();
    let msg = Message::parse_slice(&uca_msg.to_vec()).unwrap();

    //let encoded_cond = cond.encode_condition();

    let result = match cond.sign_secp256k1(&secret_key, &msg) {
        Ok(()) => (),
        Err(e) => return Err(JsValue::from_str("could sign cc")),
    };

    let js_signed_cond = make_js_cond(cond)?;

    //let js_cond = js_sys::Object::new();
    //js_sys::Reflect::set(&js_cond, &JsValue::from_str("type"), &JsValue::from_str("secp256k1-sha-256"))?;

    //Ok(JsValue::from_str(&String::from_utf8_lossy(&encoded_cond)))
    //Ok(JsValue::from_str(&hex::encode(&encoded_cond)))
    //Ok(JsValue::from_str(&hex::encode(&encoded_cond)))
    //Ok(Uint8ClampedArray::from(encoded_cond.as_slice()))
    //let jsthreshold = js_sys::Reflect::get(&target, "Threshold")?;
    //Ok(js_signed_cond)
    Ok(JsValue::from(js_signed_cond))
}

#[wasm_bindgen]
pub fn js_read_ccondition_binary(js_bin: &Uint8ClampedArray) -> Result<JsValue, JsValue> 
{
    console_log::init_with_level(Level::Debug);
    
    //let cond = decode_condition(&js_bin.to_vec()).unwrap();
    let cond: Condition = match decode_condition(&js_bin.to_vec()) {
            Ok(c) => c,
            Err(e) => return Err(JsValue::from_str("could not decode condition")),
        };
    let js_cond = make_js_cond(cond)?;

    //Ok(JsValue::from_str(&String::from_utf8_lossy(&encoded_cond)))
    //Ok(JsValue::from_str(&hex::encode(&encoded_cond)))
    //Ok(JsValue::from_str(&hex::encode(&encoded_cond)))
    Ok(js_cond)
    //let jsthreshold = js_sys::Reflect::get(&target, "Threshold")?;
}

//}
