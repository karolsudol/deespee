pub fn render_ad_markup(campaign_name: &str, user_id: &str, tracking_params: &str) -> String {
    format!(
        "<html>\
           <body style='margin:0;padding:0;'>\
             <div style='width:300px;height:250px;background:#f0f0f0;display:flex;align-items:center;justify-content:center;flex-direction:column;border:1px solid #ccc;'>\
               <h2 style='margin:0;'>{}</h2>\
               <p style='font-size:12px;'>Targeted for: {}</p>\
               <a href='http://localhost:8003/c?{}' target='_blank' style='display:inline-block;margin-top:10px;padding:10px 20px;background:#007bff;color:white;text-decoration:none;border-radius:5px;'>\
                 Click Here\
               </a>\
               <img src='http://localhost:8003/i?{}' width='1' height='1' style='display:none;' />\
               <script>\
                 (function() {{\
                   var tracked = false;\
                   var observer = new IntersectionObserver(function(entries) {{\
                     entries.forEach(function(entry) {{\
                       if (entry.isIntersecting && entry.intersectionRatio >= 0.5 && !tracked) {{\
                         setTimeout(function() {{\
                           /* Check again after 1s for IAB standard */\
                           if (!tracked) {{\
                             fetch('http://localhost:8003/v?{}');\
                             tracked = true;\
                           }}\
                         }}, 1000);\
                       }}\
                     }});\
                   }}, {{ threshold: [0.5] }});\
                   observer.observe(document.body);\
                 }})();\
               </script>\
             </div>\
           </body>\
         </html>",
        campaign_name, user_id, tracking_params, tracking_params, tracking_params
    )
}
