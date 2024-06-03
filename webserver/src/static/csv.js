function parse(text) {
  let state = 0;
  let idx = 0;
  let current = '';
  let curr_row = [];
  let rows = [];
  
  while(idx < text.length) {
    switch (text[idx]) {
      case '\\':
        current += text[idx++];
        break;

      case '"':
        if(current.length == 0) {
          while(text.length > idx && text[++idx] != '"')
            current += text[idx];
        }
        break;

      case ',':
        if (/^\d+(\.\d+)?$/.test(current)) {
          let asnum = parseFloat(current);
          curr_row.push(asnum);
        } else {
          curr_row.push(current);
        }
        current = '';
        break;

      case '\n':
        curr_row.push(current);
        current = '';
        rows.push(curr_row);
        curr_row = [];
        break;

      default:
        current += text[idx];
        break;
    }
    idx++;
  }
  return rows;
}

export default parse;
