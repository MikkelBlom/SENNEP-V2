const { SerialPort } = require('serialport');

async function listPorts() {
  try {
    const ports = await SerialPort.list();
    console.log('Available Serial Ports:');
    if (ports.length === 0) {
      console.log('No serial ports found.');
      return;
    }
    
    ports.forEach(port => {
      console.log(`- ${port.path}`);
      if (port.manufacturer) console.log(`  Manufacturer: ${port.manufacturer}`);
      if (port.serialNumber) console.log(`  Serial Number: ${port.serialNumber}`);
      if (port.vendorId) console.log(`  Vendor ID: ${port.vendorId}`);
      if (port.productId) console.log(`  Product ID: ${port.productId}`);
      console.log('---');
    });
  } catch (error) {
    console.error('Error listing ports:', error);
  }
}

listPorts();
