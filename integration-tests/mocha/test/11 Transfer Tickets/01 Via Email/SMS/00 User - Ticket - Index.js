const supertest = require('supertest');
const expect = require('chai').expect;
const mocha = require('mocha');
const tv4 = require('tv4');
const fs = require('fs');
const pm = require('../../../pm')

const baseUrl = supertest(pm.environment.get('server'));

const apiEndPoint = '/tickets?query=';


var response;
var responseBody;


const post = async function (request_body) {
    return baseUrl
        .post(pm.substitute(apiEndPoint))
        .set('Accept', 'application/json')
        .set('Content-Type', 'application/json')
        .set('Authorization', pm.substitute('Bearer {{user_token}}'))

        .send(pm.substitute(request_body));
};

const get = async function (request_body) {
    return baseUrl
        .get(pm.substitute(apiEndPoint))

        .set('Authorization', pm.substitute('Bearer {{user_token}}'))

        .set('Accept', 'application/json')
        .send();
};

let requestBody = ``;


describe('User - Ticket - Index', function () {
    before(async function () {
        response = await get(requestBody);
        console.log(response.request.header);
        console.log(response.request.url);
        console.log(response.request._data);
        console.log(response.request.method);
        responseBody = JSON.stringify(response.body);
        //console.log(pm);
        console.log(response.status);
        console.log(responseBody);
    });

    after(async function () {
        // add after methods

        let json = JSON.parse(responseBody);

        pm.environment.set("ticket1_id", json.data[0][1][0].id);
        pm.environment.set("ticket2_id", json.data[0][1][1].id);
        pm.environment.set("ticket3_id", json.data[0][1][2].id);
        pm.environment.set("ticket4_id", json.data[0][1][3].id);


    });

    it("should be 200", function () {
        expect(response.status).to.equal(200);
    })


});

            
