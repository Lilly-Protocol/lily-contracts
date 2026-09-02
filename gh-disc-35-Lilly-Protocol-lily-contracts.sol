// test/helpers/assertEvents.js
const { expect } = require('chai');

function assertEventEmitted(receipt, eventName, contractAddress, expectedTopics = {}, expectedData = {}) {
  const event = receipt.logs.find(log => 
    log.address === contractAddress && 
    log.topics[0] === web3.utils.keccak256(eventName + '(' + getEventSignature(eventName) + ')')
  );
  
  expect(event).to.exist(`Event ${eventName} was not emitted`);
  
  // Verify indexed topics (first topic is event signature, rest are indexed params)
  const eventABI = getEventABI(eventName);
  const indexedParams = eventABI.inputs.filter(input => input.indexed);
  
  indexedParams.forEach((param, i) => {
    const expectedTopic = expectedTopics[param.name];
    if (expectedTopic !== undefined) {
      const actualTopic = event.topics[i + 1]; // +1 because topics[0] is signature
      expect(actualTopic).to.equal(
        typeof expectedTopic === 'string' && expectedTopic.startsWith('0x') ? 
        expectedTopic.toLowerCase() : 
        web3.utils.padLeft(web3.utils.toHex(expectedTopic), 64)
      );
    }
  });
  
  // Verify non-indexed data
  if (Object.keys(expectedData).length > 0) {
    const decodedData = web3.eth.abi.decodeParameters(
      eventABI.inputs.filter(input => !input.indexed).map(input => input.type),
      event.data
    );
    
    Object.entries(expectedData).forEach(([param, expectedValue]) => {
      expect(decodedData[param]).to.equal(expectedValue);
    });
  }
  
  return event;
}

function getEventSignature(eventName) {
  // This would need to be implemented based on actual event definitions
  // For now, return empty string as placeholder
  return '';
}

function getEventABI(eventName) {
  // This would need to be implemented to fetch event ABI from contract
  // For now, return placeholder
  return {
    inputs: []
  };
}

module.exports = {
  assertEventEmitted
};